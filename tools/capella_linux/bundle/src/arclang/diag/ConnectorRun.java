package arclang.diag;

import java.io.FileInputStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.Map.Entry;
import java.util.Properties;

import org.eclipse.core.resources.IFile;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.NullProgressMonitor;
import org.eclipse.core.runtime.Status;
import org.eclipse.emf.common.util.URI;
import org.eclipse.emf.ecore.EObject;
import org.eclipse.emf.ecore.resource.Resource;
import org.eclipse.emf.ecore.resource.ResourceSet;
import org.eclipse.emf.ecore.resource.impl.ResourceSetImpl;
import org.eclipse.sirius.business.api.session.Session;
import org.eclipse.sirius.business.api.session.SessionManager;
import org.polarsys.capella.core.commandline.core.ui.AbstractWorkbenchCommandLine;

import org.eclipse.capella.simulink.core.model.mlinfra.PrimitiveValue;
import org.eclipse.capella.simulink.core.model.mlinfra.Value;
import org.eclipse.capella.simulink.core.model.mloperation.MloperationFactory;
import org.eclipse.capella.simulink.core.model.mloperation.Operation;
import org.eclipse.capella.simulink.m2m.capella.CatalogHelper;
import org.eclipse.capella.simulink.m2m.capella.launcher.OperationRunnable;
import org.eclipse.capella.simulink.transformation.launcher.GeneratorLauncher;

/**
 * Runs the Capella-Simulink connector on the input aird as its own tests do
 * (M2mTests.performTest, TransformationTests.getStatements): the configuration
 * (project, strategy, elements, options) in $CSC_CONFIG, the script written to $CSC_OUT.
 */
public class ConnectorRun extends AbstractWorkbenchCommandLine {
  @Override
  protected IStatus executeWithinWorkbench() {
    String out = System.getenv("CSC_OUT");
    String text;
    try {
      text = run();
    } catch (Throwable t) {
      StringWriter w = new StringWriter();
      t.printStackTrace(new PrintWriter(w));
      System.err.println("CSC failed: " + w);
      text = "CSC-ERROR " + t;
    }
    try {
      Files.write(Paths.get(out), text.getBytes(StandardCharsets.UTF_8));
    } catch (Exception e) {
      System.err.println("CSC cannot write " + out + ": " + e);
    }
    return Status.OK_STATUS;
  }

  private String run() throws Exception {
    Properties props = new Properties();
    try (FileInputStream in = new FileInputStream(System.getenv("CSC_CONFIG"))) {
      props.load(in);
    }
    List<IFile> airds = getAirdFilesFromInput();
    IFile aird = airds.get(0);
    Session session = SessionManager.INSTANCE.getSession(URI.createPlatformResourceURI(aird.getFullPath().toString(), false), new NullProgressMonitor());
    session.open(new NullProgressMonitor());

    org.eclipse.capella.simulink.core.model.mloperation.description.Operation description = CatalogHelper.HELPER.getCatalog().getOperation(props.getProperty("strategy"));
    Operation operation = MloperationFactory.eINSTANCE.createOperation();
    operation.setKind(description.getOperationkind().getId());
    CatalogHelper.HELPER.setDefaultOptions(operation);
    for (Entry<String, Value> entry : operation.getMainConfiguration().getParameters()) {
      Object value = props.get(entry.getKey());
      if (value instanceof String) {
        ((PrimitiveValue) entry.getValue()).setSpecification((String) value);
      }
    }
    for (String id : props.getProperty("elements").trim().split("\\s+")) {
      operation.getTargets().add(element(session, id));
    }
    new OperationRunnable(operation).run(new NullProgressMonitor());

    String mlcore = aird.getProject().getFile(aird.getProject().getName()).getLocation().toString() + ".mlcore";
    ResourceSet rs = new ResourceSetImpl();
    GeneratorLauncher launcher = new GeneratorLauncher();
    launcher.configure(rs);
    String generated;
    try {
      generated = launcher.generate(rs, mlcore);
    } catch (Exception e) {
      generated = e.getMessage();
    }
    if (System.getenv("CSC_MLCORE") != null) {
      Files.copy(Paths.get(mlcore), Paths.get(System.getenv("CSC_MLCORE")), java.nio.file.StandardCopyOption.REPLACE_EXISTING);
    }
    return generated;
  }

  private static EObject element(Session session, String id) {
    for (Resource res : new ArrayList<>(session.getSemanticResources())) {
      EObject found = res.getEObject(id);
      if (found != null) {
        return found;
      }
    }
    for (Resource res : new ArrayList<>(session.getAllSessionResources())) {
      EObject found = res.getEObject(id);
      if (found != null) {
        return found;
      }
    }
    throw new IllegalStateException("Cannot find element: " + id);
  }
}
