package arclang.diag;

import org.eclipse.core.runtime.IStatus;
import org.polarsys.capella.core.validation.commandline.ValidationCommandLine;

/** Same as Capella's validation command line, but prints every child status and its exception. */
public class DiagValidation extends ValidationCommandLine {
  @Override
  protected IStatus executeWithinWorkbench() {
    IStatus status;
    try {
      status = super.executeWithinWorkbench();
    } catch (Throwable t) {
      System.err.println("DIAG uncaught: " + t);
      t.printStackTrace();
      throw t;
    }
    print(status, "");
    return status;
  }

  static void print(IStatus s, String indent) {
    System.err.println("DIAG " + indent + "[" + s.getSeverity() + "] " + s.getPlugin() + ": " + s.getMessage());
    if (s.getException() != null) s.getException().printStackTrace();
    for (IStatus c : s.getChildren()) print(c, indent + "  ");
  }
}
