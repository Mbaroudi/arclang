// What the AI is told ArcLang and the workspace it runs in can do, so that it
// guides the engineer instead of answering from what it guesses: how to
// simulate with OpenModelica, generate embedded code, run twins..., which
// command to run, and whether it runs here (the desktop, arclang.fr signed
// in, arclang.fr anonymous). No VS Code here: the unit tests run it.

/** Where the extension runs. */
export type Workspace = 'desktop' | 'online' | 'online-anonymous';

interface Capability {
  what: string;
  /** How to do it: the model's side and the command (palette title or button). */
  how: string;
  /** Where it runs; absent: everywhere. */
  where?: { desktop?: string; online?: string; anonymous?: string };
}

const SIGN_IN = 'needs the visitor to sign in on arclang.fr (free): then it runs on the server';

const CAPABILITIES: Capability[] = [
  {
    what: 'Architecture, diagrams and checks',
    how: 'The Arcadia layers as text; the workbench beside the source draws every view (System, Logical, Physical, Requirements, Action Flow, Interconnection...). The status bar runs Check and Gate; "ArcLang: Run the production gate" says what blocks a release (untraced requirements, incomplete interfaces, failed or missing verdicts).',
  },
  {
    what: 'Simulation with OpenModelica (verdicts on requirements)',
    how: `1. Give the port types a physics: type "Power" { physics: electrical } (the physics are signal, electrical, thermal, mechanical_translational, mechanical_rotational; fluid is not exported yet). 2. Nodes of physical_architecture carry the parameters as quantities (energy: 75 kWh, mass: 420 kg). 3. The behaviour is a Modelica package next to the model (physics/X.mo) extending the structure the compiler generates ("ArcLang: Export Modelica" writes it). 4. A test_case says what to run and what must hold:
   test_case "Fast charge" { id: "TC-1" verifies: ["REQ-1"] method: "analysis"
     simulate: { model: "X.Charge" stop_time: 90 min sources: ["physics/X.mo"] }
     expects: ["REQ-1: max(\\"PN-CELLS\\".core.T) <= 55 °C"] }
   (sweep: [...] varies parameters; a Monte Carlo case draws them from laws.) 5. Run "ArcLang: Run simulations" (the Simulate button of the status bar): each case is simulated, its expects judged, the verdict goes to the requirement and to the gate. "ArcLang: Replay a simulation on the diagram", "ArcLang: Show Monte Carlo results", "ArcLang: Open the Modelica drawing".`,
    where: {
      desktop: 'runs with OpenModelica on this computer ("ArcLang: Set up OpenModelica" installs or finds it; else ArcLang Studio in Docker)',
      online: 'runs on the server (OpenModelica, the Modelica Standard Library and the libraries OpenModelica supports fully), counted in the visitor\'s quota',
      anonymous: SIGN_IN,
    },
  },
  {
    what: 'Modelica code',
    how: 'You may draft the Modelica package (models extending the generated structure, equations balanced). Say it is checked only when the simulation runs it.',
  },
  {
    what: 'State machines, scenarios, functional chains step by step',
    how: '"ArcLang: Run… (state machine, scenario or functional chain, in the debugger)": breakpoints, events fired by hand, variables watched.',
  },
  {
    what: 'Embedded code from the behaviour',
    how: 'State machines with variables, guards, actions and equations (data-flow blocks too) become C (MISRA C 2012), VHDL for an FPGA node, a Zephyr application, an FMU (FMI 2.0 co-simulation): "ArcLang: Export (SysML v2, Capella, ReqIF, C headers, ...)" with the format c, vhdl, zephyr or fmu. A test_case with execute: { inputs... } is run on the generated code and compared with the model: SIL (arclang sil), on the target processor in Renode (arclang pil), as VHDL in GHDL (arclang hdl), on a SoC (arclang soc), in OMSimulator (arclang cosim). "ArcLang: Realization ladder: what each stage says of each requirement" shows, per requirement, which stage passed.',
    where: {
      desktop: 'export everywhere; sil, pil, hdl, soc and cosim run with the tools on this computer or in ArcLang Studio (Docker), started by the compiler itself',
      online: 'the export (C, VHDL, Zephyr, FMU description) runs on the server; compiling and running the code (sil, pil, hdl, soc, cosim) needs the desktop: VS Code with the ArcLang extension, or ArcLang Studio',
      anonymous: `the export ${SIGN_IN}; running the code needs the desktop`,
    },
  },
  {
    what: 'Digital twins',
    how: 'A twin "Name" { ... } block runs the physics on measured data (CSV, Parquet) and judges it like a simulation: "ArcLang: Run the digital twins (the physics driven by measured data)". Live sources (MQTT, OPC UA, Kafka, HTTP): "ArcLang: Watch a digital twin live (MQTT, OPC UA, Kafka, HTTP)".',
    where: {
      desktop: 'both, with OpenModelica',
      online: 'on recorded data, on the server; live sources need the desktop (an online job has no network)',
      anonymous: `on recorded data, ${SIGN_IN}; live sources need the desktop`,
    },
  },
  {
    what: 'Impact, safety, tables, documents',
    how: '"ArcLang: Impact analysis: what if… (computed on a copy, nothing written)", "ArcLang: Safety report (hazards, FMEA, fault trees)", "ArcLang: Open a table of the model" (coverage, CESAM grid, schedule...), "ArcLang: Generate a document (design review or template)".',
    where: { online: 'on the server', anonymous: SIGN_IN },
  },
  {
    what: 'Exchange with other tools',
    how: '"ArcLang: Import a model (Capella, SysML v2, ReqIF, API server)…", "ArcLang: Save the model as… (SysML v2, Capella, ReqIF, ArcLang copy)", "ArcLang: Export the diagrams to PowerPoint (.pptx)…" or PDF.',
    where: { online: 'on the server', anonymous: SIGN_IN },
  },
  {
    what: 'Versions',
    how: '"ArcLang: Timeline: travel in the model\'s history (diagram at any commit, restore, snapshot, branch)", branches, semantic diff and merge by element, baselines.',
    where: { online: 'every save is a version of the visitor\'s repository', anonymous: 'needs the visitor to sign in (the work then becomes a git repository)' },
  },
];

const PLACE: Record<Workspace, string> = {
  desktop: 'VS Code on the engineer\'s computer, with the arclang compiler.',
  online: 'the browser, on arclang.fr, signed in: the compiler runs in the page, and what it cannot run there (simulations, gate, exports, imports, tables, history...) runs on the server.',
  'online-anonymous': 'the browser, on arclang.fr, not signed in: the compiler runs in the page (check, diagrams, edits); simulations and the commands the page cannot run need a free account.',
};

/** The guide given to the AI: what ArcLang does, how, and what runs where the engineer is. */
export function capabilitiesGuide(workspace: Workspace): string {
  const key = workspace === 'desktop' ? 'desktop' : workspace === 'online' ? 'online' : 'anonymous';
  const lines = CAPABILITIES.map((c) => {
    const where = c.where?.[key];
    return `- ${c.what}: ${c.how}${where ? `\n  Here: ${where}.` : ''}`;
  });
  return `# What ArcLang can do, and how (the engineer works in ${PLACE[workspace]})
${lines.join('\n')}

How to use this guide:
- You are the engineer's guide: they should not need a training. When they ask whether something is possible, answer from this guide (never "no" for something listed here), then give the concrete steps: what the model lacks for it (name the elements), the command to run, and whether it runs here or where.
- When the model lacks what a capability needs (physics on the port types, a test_case with simulate: and expects:, a state machine with execute: cases...), say what to add and offer to write it.
- Name commands by their exact title as above. Never invent a command or a menu.`;
}
