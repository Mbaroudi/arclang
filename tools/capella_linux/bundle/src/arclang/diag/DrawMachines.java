package arclang.diag;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

import org.eclipse.core.resources.IFile;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.NullProgressMonitor;
import org.eclipse.core.runtime.Status;
import org.eclipse.emf.common.util.TreeIterator;
import org.eclipse.emf.common.util.URI;
import org.eclipse.emf.ecore.EObject;
import org.eclipse.emf.ecore.resource.Resource;
import org.eclipse.emf.transaction.RecordingCommand;
import org.eclipse.emf.transaction.TransactionalEditingDomain;
import org.eclipse.gmf.runtime.diagram.ui.actions.ActionIds;
import org.eclipse.gmf.runtime.diagram.ui.editparts.DiagramEditPart;
import org.eclipse.gmf.runtime.diagram.core.preferences.PreferencesHint;
import org.eclipse.sirius.diagram.ui.tools.api.part.DiagramEditPartService;
import org.eclipse.gmf.runtime.diagram.ui.requests.ArrangeRequest;
import org.eclipse.gmf.runtime.notation.Diagram;
import org.eclipse.sirius.business.api.dialect.DialectManager;
import org.eclipse.sirius.business.api.session.Session;
import org.eclipse.sirius.business.api.session.SessionManager;
import org.eclipse.sirius.diagram.DDiagram;
import org.eclipse.sirius.diagram.business.api.refresh.CanonicalSynchronizerFactory;
import org.eclipse.sirius.diagram.business.api.refresh.DiagramCreationUtil;
import org.eclipse.sirius.viewpoint.DRepresentation;
import org.eclipse.sirius.viewpoint.DRepresentationDescriptor;
import org.eclipse.sirius.viewpoint.description.RepresentationDescription;
import org.eclipse.swt.widgets.Display;
import org.eclipse.swt.widgets.Shell;
import org.polarsys.capella.core.commandline.core.ui.AbstractWorkbenchCommandLine;
import org.polarsys.capella.core.data.capellacommon.Region;
import org.polarsys.capella.core.data.capellacommon.StateMachine;

/**
 * Draws every state machine of the input aird as Capella does: removes the
 * Mode State Machine diagrams it has, creates one per top-level region
 * ("[MSM] machine" or "[MSM] machine - region"), lets Sirius make its GMF
 * notation, arranges it all (GMF's Arrange All, offscreen) and saves.
 */
public class DrawMachines extends AbstractWorkbenchCommandLine {
  private static final String MSM = "Mode State Machine";

  @Override
  protected IStatus executeWithinWorkbench() {
    try {
      for (IFile aird : getAirdFilesFromInput()) {
        draw(aird);
      }
    } catch (Throwable t) {
      System.err.println("DRAW failed: " + t);
      t.printStackTrace();
      return new Status(IStatus.ERROR, "arclang.diag", "draw failed", t);
    }
    return Status.OK_STATUS;
  }

  private void draw(IFile aird) throws Exception {
    Session session = SessionManager.INSTANCE.getSession(URI.createPlatformResourceURI(aird.getFullPath().toString(), true), new NullProgressMonitor());
    session.open(new NullProgressMonitor());
    TransactionalEditingDomain domain = session.getTransactionalEditingDomain();

    // The machines' top-level regions.
    List<Region> regions = new ArrayList<>();
    for (Resource r : new ArrayList<>(session.getSemanticResources())) {
      for (TreeIterator<EObject> it = r.getAllContents(); it.hasNext();) {
        EObject o = it.next();
        if (o instanceof StateMachine) {
          regions.addAll(((StateMachine) o).getOwnedRegions());
        }
      }
    }
    List<Diagram> drawn = new ArrayList<>();
    domain.getCommandStack().execute(new RecordingCommand(domain) {
      @Override
      protected void doExecute() {
        for (DRepresentationDescriptor d : new ArrayList<>(DialectManager.INSTANCE.getAllRepresentationDescriptors(session))) {
          if (MSM.equals(d.getDescription().getName())) {
            DialectManager.INSTANCE.deleteRepresentation(d, session);
          }
        }
        for (Region region : regions) {
          StateMachine machine = (StateMachine) region.eContainer();
          String name = machine.getOwnedRegions().size() > 1 ? "[MSM] " + machine.getName() + " - " + region.getName() : "[MSM] " + machine.getName();
          RepresentationDescription description = null;
          Collection<RepresentationDescription> available = DialectManager.INSTANCE.getAvailableRepresentationDescriptions(session.getSelectedViewpoints(false), region);
          for (RepresentationDescription rd : available) {
            if (MSM.equals(rd.getName())) {
              description = rd;
            }
          }
          if (description == null) {
            System.err.println("DRAW no MSM description for " + name + " among " + available.size());
            continue;
          }
          DRepresentation rep = DialectManager.INSTANCE.createRepresentation(name, region, description, session, new NullProgressMonitor());
          DiagramCreationUtil util = new DiagramCreationUtil((DDiagram) rep);
          if (!util.findAssociatedGMFDiagram()) {
            util.createNewGMFDiagram();
          }
          Diagram gmf = util.getAssociatedGMFDiagram();
          CanonicalSynchronizerFactory.INSTANCE.createCanonicalSynchronizer(gmf).synchronize();
          drawn.add(gmf);
          System.err.println("DRAW created " + name);
        }
      }
    });
    Display display = Display.getDefault();
    for (Diagram gmf : drawn) {
      Shell shell = new Shell(display);
      // As the connector reads a diagram, then as Sirius arranges one at its first opening (LayoutUtil.arrange).
      DiagramEditPart part = new DiagramEditPartService().createDiagramEditPart(gmf, shell, PreferencesHint.USE_DEFAULTS);
      part.getRoot().refresh();
      part.getFigure().validate();
      part.getRoot().refresh();
      part.getViewer().flush();
      ArrangeRequest request = new ArrangeRequest(ActionIds.ACTION_ARRANGE_ALL);
      List<Object> parts = new ArrayList<>();
      parts.add(part);
      request.setPartsToArrange(parts);
      part.performRequest(request);
      while (display.readAndDispatch()) {
        // the arrange's deferred work
      }
      System.err.println("DRAW arranged " + ((DDiagram) gmf.getElement()).getName());
      part.getFigure().validate();
      part.deactivate();
      shell.dispose();
    }
    session.save(new NullProgressMonitor());
    System.err.println("DRAW saved " + drawn.size() + " diagrams");
  }
}
