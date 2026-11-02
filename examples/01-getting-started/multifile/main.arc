// Multi-file model: each team owns its fragment, the root file assembles
// them. Paths are resolved relative to THIS file. The traces below reference
// elements from BOTH fragments — they only resolve after the merge, which is
// exactly what makes this a model assembly and not a text include.

import "requirements.arc"
import "logical.arc"

model MultiFileDemo {
  version: "1.0"
}

trace "LC-MF-001" satisfies "REQ-MF-001" { rationale: "braking command path" }
trace "LC-MF-002" satisfies "REQ-MF-002" { rationale: "override detection" }
