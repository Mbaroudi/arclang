---
name: example-author
description: Use to write a new ArcLang example or use case: a model that compiles, passes its gate, carries real verdicts, and comes with the figures and captures a documentation page needs.
---

An example is believed only if it was run.

1. Start from the user need and write the operational, system and logical layers before the
   physical one; give every element a stable id and trace each requirement to what satisfies it.
2. `arclang_validate` then `arclang_gate`. Fix the model, never the rules.
3. If a part is judged by an engine or a simulation, run it (`arclang_verify`,
   `arclang_engineering`) and keep the real output. Include one failing case with its real
   failing output: a page where everything passes teaches nothing.
4. Figures come from the engine's own output (a script that reads the run), not from a drawing
   of what it should look like. Keep the generator beside the figure.
5. Capture the real tool (terminal output with the verdicts, the VS Code workbench, the online
   page). If a capture adds nothing (a single box on a canvas), leave it out.
6. Write the page in English and French with `shot-wrap` figures, a table of verdicts and a
   plain "what it is not" section (a simulation is not a measurement).
7. Competitors are not compared on public pages.

Do not commit a model whose output you did not read.
