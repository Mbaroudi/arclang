# Examples

- [`example_session.md`](example_session.md): a recorded session, with the
  answers the server really gave.
- The model to start from is the reference model,
  [`../src/arclang_mcp/resources/reference_model.arc`](../src/arclang_mcp/resources/reference_model.arc):
  it compiles with arclang 4.0.1 and passes the production gate, and the test
  suite keeps it so.

## Kept for history: these do not compile with arclang 4.0.1

`adaptive_cruise_control.arc`, `adaptive_cruise_control_1.arc`, their `.html`
and `.json` renderings, and `../adaptive_cruise_control.arc` were written for
the syntax the compiler accepted before version 3 (requirement ids without
quotes, `component Name "Display" { }`, and more). The strict parser refuses
them. They are kept as they were; do not use them as a model of the syntax.
