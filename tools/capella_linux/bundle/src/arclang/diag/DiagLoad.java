package arclang.diag;

import java.util.List;
import org.eclipse.core.resources.IFile;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.NullProgressMonitor;
import org.eclipse.core.runtime.Status;
import org.eclipse.emf.common.util.URI;
import org.eclipse.emf.ecore.EObject;
import org.eclipse.emf.ecore.resource.Resource;
import org.eclipse.sirius.business.api.session.Session;
import org.eclipse.sirius.business.api.session.SessionManager;
import org.polarsys.capella.core.commandline.core.ui.AbstractWorkbenchCommandLine;

/** Opens each input aird and prints every resource, its roots and its load errors/warnings. */
public class DiagLoad extends AbstractWorkbenchCommandLine {
  @Override
  protected IStatus executeWithinWorkbench() {
    List<IFile> airds = getAirdFilesFromInput();
    System.err.println("DIAG airds: " + airds);
    for (IFile f : airds) {
      URI uri = URI.createPlatformResourceURI(f.getFullPath().toString(), true);
      try {
        Session s = SessionManager.INSTANCE.getSession(uri, new NullProgressMonitor());
        System.err.println("DIAG session " + s + " open=" + s.isOpen());
        Object prj = org.polarsys.capella.core.sirius.ui.helper.SessionHelper.getCapellaProject(s);
        System.err.println("DIAG getCapellaProject -> " + prj);
        for (Resource r : s.getTransactionalEditingDomain().getResourceSet().getResources()) {
          System.err.println("DIAG resource " + r.getURI() + " loaded=" + r.isLoaded() + " roots=" + r.getContents().size());
          for (EObject o : r.getContents()) System.err.println("DIAG   root " + o.eClass().getName() + " proxy=" + o.eIsProxy());
          for (Resource.Diagnostic d : r.getErrors()) System.err.println("DIAG   ERROR " + d.getMessage() + " @" + d.getLine() + ":" + d.getColumn());
          int n = 0;
          for (Resource.Diagnostic d : r.getWarnings()) { if (n++ < 20) System.err.println("DIAG   WARN " + d.getMessage() + " @" + d.getLine()); }
        }
        System.err.println("DIAG semantic: " + s.getSemanticResources());
        for (Resource r : s.getSemanticResources()) {
          System.err.println("DIAG before-load " + r.getURI() + " loaded=" + r.isLoaded() + " roots=" + r.getContents().size() + (r.getContents().isEmpty() ? "" : " first=" + r.getContents().get(0).eClass().getName()) + " errors=" + r.getErrors().size());
          for (Resource.Diagnostic d : r.getErrors()) System.err.println("DIAG   PRE-ERROR " + d.getMessage());
          try { if (!r.isLoaded()) r.load(null); } catch (Throwable t) { System.err.println("DIAG load failed " + t); t.printStackTrace(); }
          System.err.println("DIAG sem " + r.getURI() + " loaded=" + r.isLoaded() + " roots=" + r.getContents().size());
          for (EObject o : r.getContents()) System.err.println("DIAG   root " + o.eClass().getName() + " " + o.eClass().getEPackage().getNsURI());
          for (Resource.Diagnostic d : r.getErrors()) System.err.println("DIAG   ERROR " + d.getMessage() + " @" + d.getLine() + ":" + d.getColumn());
          int n = 0;
          for (Resource.Diagnostic d : r.getWarnings()) { if (n++ < 20) System.err.println("DIAG   WARN " + d.getMessage() + " @" + d.getLine()); }
        }
      } catch (Throwable t) {
        System.err.println("DIAG failed: " + t);
        t.printStackTrace();
      }
    }
    return Status.OK_STATUS;
  }
}
