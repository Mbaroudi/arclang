// Fragment: logical architecture owned by the architecture team.
// Compiles standalone AND as part of main.arc (import merge).

architecture logical {
  component "Brake Controller" {
    id: "LC-MF-001"
    function "Compute braking command"
  }
  component "Override Monitor" {
    id: "LC-MF-002"
    function "Detect driver override"
  }
  component_exchange "override signal" {
    from_port: "LC-MF-002"
    to_port: "LC-MF-001"
  }
}
