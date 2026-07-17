// Fragment: requirements owned by the systems-engineering team.
// Compiles standalone AND as part of main.arc (import merge).

requirements safety {
  req "REQ-MF-001" "Braking latency" {
    description: "The subsystem shall brake within 100 ms"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-MF-002" "Driver override" {
    description: "The driver shall be able to override automatic braking"
    priority: "Medium"
  }
}
