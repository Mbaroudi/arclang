// What the AI is told ArcLang and the workspace it runs in can do, so that it
// guides the engineer through everything the site and the documentation
// show, instead of answering from what it guesses: each topic with its page
// of the documentation, how to do it (the model's side, the command palette
// title, the `arclang` command) and whether it runs here (the desktop,
// arclang.fr signed in, arclang.fr anonymous). No VS Code here: the unit
// tests run it, and check that every page of the documentation and every
// command named exists.

/** Where the extension runs. */
export type Workspace = 'desktop' | 'online' | 'online-anonymous';

type Place = 'desktop' | 'online' | 'anonymous';

export interface Topic {
  what: string;
  /** Its page under https://arclang.fr/docs/ (a slug), or a file of the repository (docs/x.md). */
  docs: string[];
  /** How to do it: the model's side and the commands. */
  how: string;
  /** Where it runs; absent: everywhere, in the page. */
  where?: Partial<Record<Place, string>>;
}

const SIGN_IN = 'needs the visitor to sign in on arclang.fr (free), then it runs on the server';
const SERVER = { online: 'on the server', anonymous: SIGN_IN };
const DESKTOP = 'VS Code with the ArcLang extension and the compiler, or ArcLang Studio (one Docker image with everything)';
/** Online, simulations start from the Simulate button of the diagram: the palette's simulation commands are the desktop's. */
const SIMULATE_ONLINE = 'click Simulate at the bottom of the diagram (the workbench beside the source: "ArcLang: Open the workbench beside the source")';
const DESKTOP_ONLY = (what: string) => ({ online: `${what} needs the desktop: ${DESKTOP}`, anonymous: `${what} needs the desktop: ${DESKTOP}` });

export const TOPICS: Topic[] = [
  // ---- Learn ----
  {
    what: 'Learning MBSE and ArcLang from zero',
    docs: ['mbse', 'tutorial', 'getting-started', 'language'],
    how: 'MBSE in 15 minutes explains the ideas on a home heating example; the tutorial builds that model from a blank page (requirements, the Arcadia levels, traces, tests, check, gate); Getting started installs the compiler; the language tour shows every construct. "ArcLang: New model…" starts one (arclang new <name> --template cesam for CESAM).',
  },
  {
    what: 'The Arcadia method (Capella\'s): operational analysis, system analysis, logical and physical architecture, EPBS',
    docs: ['arcadia', 'language', 'diagrams'],
    how: 'One block per level (operational_analysis, system_analysis, logical_architecture, physical_architecture, epbs) with its elements; "ArcLang: New layer…" adds one; "ArcLang: Transition plan (what each Arcadia step would create)" and "ArcLang: Apply an Arcadia transition step" create the next level from the previous one (arclang transition oa-to-sa | sa-to-la | la-to-pa).',
  },
  {
    what: 'SysML v2 concepts (parts, ports, connections, actions, requirements, verifications, states)',
    docs: ['sysml-v2-modeling', 'sysml-v2'],
    how: 'Each SysML v2 concept and the ArcLang element it maps to; .sysml files open in the workbench; import and export below.',
  },
  {
    what: 'The CESAM method: three visions (operational, functional, constructional) × six aspects, on the same Arcadia model',
    docs: ['cesam', 'cesam-tutorial'],
    how: 'model X { method: "cesam" }; a requirements stakeholder { ... } block holds stakeholder "Resident" { id: "STK-RES" category: "user" interest: "..." }, each of their needs as a concern: concern "Keyless entry" { id: "NEED-1" stakeholders: ["STK-RES"] }, and the requirements that frame them req "STK-REQ-1" "..." { frames: ["NEED-1"] }; state machines with kind: "lifecycle" | "functional_modes" | "technical_configurations" (a state realizes: a state of another, context:, during:); vision: places a requirement, scenario or machine in a vision; capability: "UC-1" on a scenario or functional chain names the use case it illustrates. "arclang new <name> --template cesam" starts a model with every view in place. "ArcLang: Open a table of the model" with the kind cesam_grid shows the grid and each view\'s maturity (arclang table --kind cesam_grid); arclang check --lint --method cesam gives the ten cesam.* rules; arclang diagram --view cesam draws the views; the export to SysML v2 has one view per CESAM view. The CESAM tutorial: nine exercises.',
    where: { online: 'here, in the playground: its compiler has CESAM', anonymous: 'here, in the playground: its compiler has CESAM', desktop: 'from release 4.13 of the compiler (before it, the compiler built from main)' },
  },
  // ---- Model ----
  {
    what: 'Quantities, units, types, specialization, constraints',
    docs: ['types-and-constraints', 'metamodel', 'grammar'],
    how: 'Values with units (20 ms, 75 kWh), checked dimensions; reusable type definitions with specialization; constraints the compiler evaluates. "ArcLang: Show the language metamodel" lists every kind and attribute; the grammar is the exact syntax.',
  },
  {
    what: 'Diagrams and the workbench (editing like Capella or draw.io)',
    docs: ['diagrams', 'vscode'],
    how: '"ArcLang: Open the workbench beside the source": every view (operational, system, logical, physical, capabilities, states, sequences, action flow, interconnection...), a palette, properties, a model tree; edits on the canvas are written to the text and compiled. "ArcLang: New diagram…", "ArcLang: Open a diagram…", views by concern (tags), layers on the canvas, "ArcLang: Choose the diagram color theme…". "ArcLang: Export the diagrams to PowerPoint (.pptx)…", "ArcLang: Export the diagrams to PDF, one page each with its title and legend…", "ArcLang: Copy the diagram as an image (for PowerPoint, Keynote, Word)".',
  },
  {
    what: 'Product lines, variants and configurations; ports holding ports, buses with any number of ends',
    docs: ['language', 'types-and-constraints'],
    how: 'configurations select the elements of one variant of the architecture; nested ports and buses are elements of the physical architecture; simulations run per configuration.',
  },
  // ---- Verify ----
  {
    what: 'Checks and the production gate (ISO 26262, DO-178C)',
    docs: ['production-gate', 'rules'],
    how: 'Check and Gate in the status bar; "ArcLang: Check the model (rules and findings)", "ArcLang: Run the production gate": PASS or FAIL with each blocker (requirement coverage and verification, hazards and ASIL consistency, timing budgets, interface completeness). "ArcLang: Show the catalog of rules" explains each rule (arclang rules).',
  },
  {
    what: 'Simulation with OpenModelica: verdicts on requirements',
    docs: ['simulation', 'docs/modelica.md'],
    how: `1. Give the port types a physics: type "Power" { physics: electrical } (the physics are signal, electrical, thermal, mechanical_translational, mechanical_rotational; fluid is not exported yet). 2. Nodes of physical_architecture carry their parameters as quantities (energy: 75 kWh, mass: 420 kg). 3. The behaviour is a self-contained Modelica package next to the model (physics/X.mo, written with write_file): one model per case (X.Charge) holding instances (pack) of models whose Real variables carry their unit (Real T(unit = "K")); the compiler also generates the structure of the architecture ("ArcLang: Export Modelica", arclang export -f modelica) for packages that extend it. 4. A test_case says what to run and what must hold:
   test_case "Fast charge" { id: "TC-1" verifies: ["REQ-1"] method: "analysis"
     simulate: { model: "X.Charge" stop_time: 90 min sources: ["physics/X.mo"] }
     expects: ["REQ-1: max(pack.cell.T) <= 55 °C"] }
   (an expects: reads a variable of the simulated model by its path from it, pack.cell.T, as the result names it; the bound has a unit of the same kind as the variable's unit, °C converting from K; a variable without unit compares with a plain number.)
   (sweep: [...] varies parameters.) 5. "ArcLang: Run simulations" (the Simulate button of the status bar; arclang verify): each case is simulated, its expects judged, the verdict goes to the requirement, the gate and the tables. Then "ArcLang: Replay a simulation on the diagram", "ArcLang: Open the Modelica drawing", "ArcLang: Export the simulation report (PDF or PowerPoint)…"; OMEdit opens the exported package.`,
    where: {
      desktop: 'with OpenModelica on this computer ("ArcLang: Set up OpenModelica" installs or finds it; else ArcLang Studio in Docker)',
      online: `${SIMULATE_ONLINE}: it runs on the server (OpenModelica 1.27.1, the Modelica Standard Library 4.1.0, and the libraries OpenModelica fully supports, such as Buildings, ThermofluidStream, PlanarMechanics, VehicleInterfaces, PowerGrids, iPSL, PhotoVoltaics: a package loads one by declaring it in annotation(uses(...))), counted in the visitor's quota. "Run simulations", "Export Modelica" and "Open the Modelica drawing" are desktop commands, not in the palette here`,
      anonymous: `sign in first (free, a link by e-mail: "Sign in to keep your work" in the status bar), then ${SIMULATE_ONLINE}: it runs on the server. "Run simulations", "Export Modelica" and "Open the Modelica drawing" are desktop commands, not in the palette here`,
    },
  },
  {
    what: 'Monte Carlo and statistics: probabilistic verdicts, sensitivity',
    docs: ['simulation'],
    how: 'A simulated case draws its parameters from laws and gives a verdict with a confidence (Clopper-Pearson), quantiles and sensitivity; "ArcLang: Show Monte Carlo results"; R or Python statistics (arclang verify --statistics r|python).',
    where: { desktop: 'with OpenModelica on this computer', online: `${SIMULATE_ONLINE}: on the server, R and Python included; the "Show Monte Carlo results" page needs the desktop`, anonymous: `${SIGN_IN}, with Simulate at the bottom of the diagram; the "Show Monte Carlo results" page needs the desktop` },
  },
  {
    what: 'Modelica code',
    docs: ['simulation', 'docs/modelica.md'],
    how: 'Write the Modelica package next to the model with write_file (self-contained: one model per case, Real variables with their unit, equations balanced); each expects: reads a variable of the simulated model by its path (plant.water.T). It is checked when the simulation runs it.',
  },
  {
    what: 'Running state machines, scenarios and functional chains step by step',
    docs: ['vscode'],
    how: '"ArcLang: Run… (state machine, scenario or functional chain, in the debugger)": breakpoints, "ArcLang: Fire an event", variables watched (arclang debug).',
  },
  // ---- Embedded ----
  {
    what: 'Embedded C from the behaviour (MISRA C 2012), software in the loop, the plant in the loop, tasks',
    docs: ['embedded-c'],
    how: 'State machines with variables, guards, actions and equations, and data-flow blocks, become neutral C: "ArcLang: Export (SysML v2, Capella, ReqIF, C headers, ...)" with the format c (arclang export -f c). An io_binding binds a machine to the I/O of a target (a task, a period). A test_case with execute: { machine: ... inputs: [...] } runs the model; arclang sil compiles the C on this computer and compares it with the model step by step; execute: { plant: ... } closes the loop on a Modelica or FMU plant; arclang schedule checks that every task meets its period.',
    where: { desktop: 'export, sil and schedule on this computer', online: `the export of the C runs on the server; compiling and running it (sil) needs the desktop: ${DESKTOP}`, anonymous: `the export of the C ${SIGN_IN}; compiling and running it (sil) needs the desktop: ${DESKTOP}` },
  },
  {
    what: 'Reverse engineering: the machines a generated C implements, and the drift of the code as built from the model',
    docs: ['docs/reverse-engineering.md'],
    how: 'arclang recover <dir> reads the state machines back from the C the compiler generated (<model>.h and <model>.c) and writes them as ArcLang; with --check the recovered model must generate that same C, line for line, or the command fails with the lines that differ: only then is the recovery proven. With --hdl <dir> (the directory arclang export -f vhdl wrote) the machines of the programmable logic are sized from its register maps and --check proves the logic too: every file of it must be generated again as it was read. arclang drift <model> <dir> compares the C as built with the model: each difference (a guard, an initial value, an equation, the time or the target of a transition) is said in the terms of the model with its line in the code, and the lines the machines do not explain are listed; every other generated file (the I/O layer, the host files, and the VHDL with --hdl <dir>) is compared line by line. With --prove (CBMC and Bitwuzla, in the image of tools/embedded/formal) one step of the code as built is proven to do what one step of the code the model generates does, from any state and for any inputs, or refuted with the state and the inputs that show it; say "proven equivalent" only when the command printed it. Code written by hand (an Arduino sketch) is tied to a machine by a correspondence declaration (machine, source, period, reads: ["digitalRead(BUTTON) is button"], writes: ["digitalWrite(LED) is led"], state: ["mode == 1 is in Heating", "millis() - since is time in Rising", "lit == 1 is led"], invariant: ["lit == 0 || lit == 1"], int: 16 for an AVR board such as the Uno) and arclang prove <model> proves it (same image): the start of the two corresponds, and one loop() and one step keep them corresponding from any corresponding states; refuted, it names the lines of the correspondence that do not hold; a model written or proposed for hand-written code is only "proven" when the command printed Proven. arclang recover <sketch.ino> --propose --with "<command>" asks a proposer (any command that reads a prompt and prints an answer) for the machine and the correspondence of a sketch, proves the answer, sends back what is refuted with a case, and writes the model only once proven; to propose a model yourself, write the machine and the correspondence, run arclang prove, read the lines that do not hold and the case, and correct until it prints Proven. Never call a recovery proven without --check, and never say the code matches the model without drift.',
    where: { desktop: 'recover and drift on this computer', online: `recover and drift need the desktop: ${DESKTOP}`, anonymous: `recover and drift need the desktop: ${DESKTOP}` },
  },
  {
    what: 'Embedded targets and processor in the loop (Zephyr, Renode)',
    docs: ['embedded-targets'],
    how: 'A target package (zephyr/litex_vexriscv): arclang export -f zephyr writes a Zephyr application; arclang pil runs the C on the target\'s processor in Renode and compares it with the model.',
    where: { desktop: 'with the tools on this computer, else the compiler starts ArcLang Studio (Docker) by itself', ...DESKTOP_ONLY('Processor in the loop (pil)') },
  },
  {
    what: 'FPGA: VHDL from the model, fixed point, synthesis, HW/SW partitioning',
    docs: ['embedded-fpga'],
    how: 'A machine allocated to a programmable-logic node becomes VHDL in fixed point derived from its ranges (arclang export -f vhdl); arclang hdl simulates it in GHDL against the bit-exact C and the model; --synth synthesises it (Yosys, nextpnr); arclang partition measures each machine as C and as VHDL to choose where it goes.',
    where: { desktop: 'with the tools on this computer, else ArcLang Studio (Docker)', online: `the export of the VHDL runs on the server; hdl, --synth and partition need the desktop: ${DESKTOP}`, anonymous: `the export of the VHDL ${SIGN_IN}; hdl, --synth and partition need the desktop: ${DESKTOP}` },
  },
  {
    what: 'Processor and logic together on a SoC, and a real board',
    docs: ['embedded-system'],
    how: 'arclang soc: one LiteX SoC (a VexRiscv running the Zephyr application, the VHDL blocks on its bus) simulated in Verilator with the plant; --board builds the bitstream for a real board (ULX3S), --port runs the cases on it.',
    where: { desktop: 'with the tools on this computer, else ArcLang Studio (Docker)', ...DESKTOP_ONLY('The SoC (soc)') },
  },
  {
    what: 'Co-simulation in OMSimulator: FMUs and SSP',
    docs: ['cosimulation'],
    how: 'arclang cosim: each case that execute:s runs as a standard FMI co-simulation: the behaviour\'s FMU (arclang export -f fmu), the plant\'s FMU (built by OpenModelica from the Modelica plant the case names in execute: { plant: ... }, or an FMU the case gives) and the inputs in an SSP package OMSimulator runs, judged against the model; --keep keeps cosim.ssp.',
    where: { desktop: 'with OMSimulator on this computer, else ArcLang Studio (Docker)', online: `${SIMULATE_ONLINE}: the test cases that execute: are co-simulated on the server in OMSimulator, in the same run as the simulations and counted once in the quota; the verdicts show in the notification and in the ArcLang Simulation log`, anonymous: `sign in first (free, a link by e-mail: "Sign in to keep your work" in the status bar), then ${SIMULATE_ONLINE}: the test cases that execute: are co-simulated on the server in OMSimulator` },
  },
  {
    what: 'The realization ladder: each requirement from simulation to SIL, PIL, HDL, SoC, board and twin',
    docs: ['realization'],
    how: '"ArcLang: Realization ladder: what each stage says of each requirement": per requirement, which stage passed, failed, is stale or never ran, the next command with Run; arclang realize climbs every missing or stale stage (--dry-run prints the plan).',
    where: { online: 'the ladder shows the verdicts kept; climbing the embedded stages needs the desktop', anonymous: 'the ladder shows the verdicts kept; simulations need to sign in, the embedded stages the desktop' },
  },
  {
    what: 'ArcLang Studio: the whole chain in one Docker image',
    docs: ['studio'],
    how: 'ghcr.io/mbaroudi/arclang-studio: VS Code in the browser with the extension, the compiler, OpenModelica and OMSimulator, Zephyr and Renode, GHDL, Yosys, nextpnr, LiteX, Verilator. On a laptop, a server or as a dev container (.devcontainer).',
    where: { online: 'it runs on your own computer or server, not on arclang.fr', anonymous: 'it runs on your own computer or server, not on arclang.fr' },
  },
  // ---- Digital twins ----
  {
    what: 'Digital twins: the physics driven by measured data',
    docs: ['simulation', 'docs/digital-twin.md'],
    how: 'A twin "Name" { ... } block runs the physical model on measured data (CSV, Parquet: drives:, compares:) and judges it with expects: like a simulation: "ArcLang: Run the digital twins (the physics driven by measured data)" (arclang twin). Live sources (MQTT, OPC UA, Kafka, HTTP), the model advancing with the data (an FMU): "ArcLang: Watch a digital twin live (MQTT, OPC UA, Kafka, HTTP)".',
    where: {
      desktop: 'both, with OpenModelica',
      online: 'on recorded data, on the server; live sources need the desktop (an online job has no network)',
      anonymous: `on recorded data, ${SIGN_IN}; live sources need the desktop`,
    },
  },
  // ---- Review and change ----
  {
    what: 'Impact of a change, what-if',
    docs: ['review-and-change', 'docs/IMPACT.md'],
    how: '"ArcLang: Impact analysis: analyze an element…", "ArcLang: Impact analysis: what if… (computed on a copy, nothing written)", "ArcLang: Impact analysis: my changes (since HEAD or main)" (arclang impact); after an edit the workbench lists its consequences and the AI proposes fixes.',
    where: SERVER,
  },
  {
    what: 'Semantic diff, review, formatting, scripted edits',
    docs: ['review-and-change'],
    how: '"ArcLang: Semantic diff against the committed version" (arclang diff: by element identity, a move is no change); "ArcLang: Format the model"; arclang set / rename / edit change one element and are refused unless the model still compiles; arclang report writes the pull-request report.',
    where: SERVER,
  },
  {
    what: 'Versions: history, branches, merges, baselines, time travel',
    docs: ['semantic-git'],
    how: '"ArcLang: Timeline: travel in the model\'s history (diagram at any commit, restore, snapshot, branch)", "ArcLang: History of an element (the commits that changed it)…", "ArcLang: Switch branch…", "ArcLang: Merge a branch into the current one (by element)…", "ArcLang: Compare the model with a branch, tag, baseline or HEAD…", "ArcLang: Create baseline…", "ArcLang: Verify a baseline…", "ArcLang: Install the semantic merge driver" (arclang merge --install).',
    where: { online: 'every save is a version of the visitor\'s repository on arclang.fr', anonymous: 'needs the visitor to sign in (the work then becomes a git repository, every save a version)' },
  },
  {
    what: 'CI/CD: the gate and the simulations in a pipeline',
    docs: ['ci-cd'],
    how: '"ArcLang: Configure CI" (arclang ci init) writes the pipeline for GitHub Actions, GitLab CI, Azure DevOps or Jenkins: every model compiled, the gate, the simulations, SARIF for code scanning (arclang sarif), a pull-request report; "ArcLang: Show the CI pipeline status".',
    where: { online: '"Configure CI" and "Show the CI pipeline status" are desktop commands; the pipeline runs in the team\'s own CI', anonymous: '"Configure CI" and "Show the CI pipeline status" are desktop commands; the pipeline runs in the team\'s own CI' },
  },
  // ---- Safety, tables, documents ----
  {
    what: 'Safety: hazards, FMEA, fault trees, ASIL',
    docs: ['production-gate'],
    how: '"ArcLang: Safety report (hazards, FMEA, fault trees)" (arclang safety); the gate checks hazard and ASIL consistency.',
    where: SERVER,
  },
  {
    what: 'Tables and matrices: coverage, traceability, allocation, N², ICD, schedule, partition, CESAM grid',
    docs: ['cli'],
    how: '"ArcLang: Open a table of the model", "ArcLang: Requirement coverage table", "ArcLang: New table…" (arclang table --kind ...); export to CSV, Markdown, HTML or Excel; a table edits the model.',
    where: SERVER,
  },
  {
    what: 'Documents: design reviews and templates (Markdown, HTML, Word)',
    docs: ['docs/documents.md'],
    how: '"ArcLang: Generate a document (design review or template)" (arclang document): a template\'s {{ ... }} directives filled from the model (values, tables, verdicts, gate).',
    where: SERVER,
  },
  {
    what: 'Traces and their review',
    docs: ['language'],
    how: 'Traces between levels and to requirements (arclang trace lists and checks them); "ArcLang: Mark every trace and verification as reviewed (suspect when an end changes)": a changed end makes a trace suspect until reviewed again.',
  },
  // ---- Exchange ----
  {
    what: 'Capella: import a native model, edit it as text, write it back byte-identically',
    docs: ['capella'],
    how: '"ArcLang: Import a Capella model…"; "ArcLang: Save the model as… (SysML v2, Capella, ReqIF, ArcLang copy)" writes the .capella back.',
    where: SERVER,
  },
  {
    what: 'SysML v2: import any model, export to the OMG pilot, the Systems Modeling API',
    docs: ['sysml-v2', 'api'],
    how: '"ArcLang: Import SysML v2 file(s) or folder…", "ArcLang: Import from a SysML v2 API server…", export with the format sysml; arclang serve exposes the compile endpoint and the Systems Modeling API (projects, commits from git, elements, queries).',
    where: { online: 'import and export on the server; arclang serve runs on your own machine', anonymous: `import and export ${SIGN_IN}; arclang serve runs on your own machine` },
  },
  {
    what: 'ReqIF (DOORS, Polarion), Simulink and System Composer, FMI 2.0, C headers, Protobuf',
    docs: ['exchange-formats', 'docs/simulink-connector.md'],
    how: '"ArcLang: Import requirements (ReqIF)…", "ArcLang: Import from DOORS (its ReqIF export)…"; "ArcLang: Export (SysML v2, Capella, ReqIF, C headers, ...)" with ReqIF, Simulink, FMI, C headers or Protobuf (arclang export -f ...), deterministic outputs.',
    where: SERVER,
  },
  // ---- Tools around ----
  {
    what: 'The command line',
    docs: ['cli'],
    how: 'arclang check, gate, diagram, export, import, diff, history, merge, restore, baseline, ci, report, sarif, impact, debug, verify, cosim, sil, pil, hdl, soc, partition, schedule, realize, twin, document, table, safety, set, rename, edit, workbench, transition, new, format, rules, metamodel, serve, lsp. Every command has --help.',
  },
  {
    what: 'AI agents: this chat, agent tools in VS Code, the MCP server',
    docs: ['vscode'],
    how: 'This chat edits through arclang edit requests the compiler checks before the engineer applies them; in VS Code, @arclang in Copilot Chat and agent tools; the MCP server (mcp-server/) gives any agent the compiler; "ArcLang: Configure AI (provider, model, suggestions)" chooses the provider or a personal API key.',
    where: { online: 'this chat on arclang.fr (free, limited) or with your own key; the MCP server runs on your own machine', anonymous: 'this chat on arclang.fr (free, limited) or with your own key; the MCP server runs on your own machine' },
  },
  {
    what: 'The playground on arclang.fr: VS Code in the browser with the compiler, examples, a guided tour',
    docs: ['tutorial', 'docs/WEB_PLAYGROUND.md'],
    how: 'arclang.fr/playground: the compiler runs in the page (WebAssembly: the model never leaves it), the examples, the guided tour; a free account adds the server (simulations, exports, imports, tables, history) and keeps the work as a git repository.',
  },
  {
    what: 'The platform, versions and stability',
    docs: ['platform', 'versioning', 'migration-4.0'],
    how: 'What the compiler is made of and what comes next; how the language is versioned (stable identities, deterministic outputs, deprecations); migrating a 3.x model to 4.0.',
  },
  {
    what: 'Training and certification, use cases',
    docs: [],
    how: 'arclang.fr/training/ (courses and certification), arclang.fr/use-cases/ (worked industrial examples, such as AI datacenter liquid cooling).',
  },
];

const PLACE: Record<Workspace, string> = {
  desktop: 'VS Code on the engineer\'s computer, with the arclang compiler.',
  online: 'the browser, on arclang.fr, signed in: the compiler runs in the page, and what it cannot run there (simulations, gate, exports, imports, tables, history...) runs on the server.',
  'online-anonymous': 'the browser, on arclang.fr, not signed in: the compiler runs in the page (check, diagrams, edits); simulations and the commands the page cannot run need a free account.',
};

const docLink = (d: string) => (d.startsWith('docs/') ? `github.com/Mbaroudi/arclang/blob/main/${d}` : `arclang.fr/docs/${d}/`);

/** The guide given to the AI: what ArcLang does, how, and what runs where the engineer is. */
export function capabilitiesGuide(workspace: Workspace): string {
  const place: Place = workspace === 'desktop' ? 'desktop' : workspace === 'online' ? 'online' : 'anonymous';
  const lines = TOPICS.map((t) => {
    const where = t.where?.[place];
    const docs = t.docs.length ? ` [${t.docs.map(docLink).join(', ')}]` : '';
    return `- ${t.what}${docs}: ${t.how}${where ? `\n  Here: ${where}.` : ''}`;
  });
  return `# What ArcLang can do, and how (the engineer works in ${PLACE[workspace]})
${lines.join('\n')}

How to use this guide:
- You are the engineer's guide: they should not need a training. When they ask whether something is possible, answer from this guide (never "no" for something listed here), then give the concrete steps: what the model lacks for it (name the elements), the command to run, and whether it runs here or where.
- When the model lacks what a capability needs (physics on the port types, a test_case with simulate: and expects:, a state machine with execute: cases, method: "cesam"...), add it with your tools in the same answer (the Modelica package with write_file), choosing sensible values and saying which.
- A test_case verifies its requirement once its outcome is known: computed (simulate:, execute:, asserts:) or recorded from a test run elsewhere (verdict: "pass" or "fail"). A case that says nothing verifies nothing, and the gate says so. Never record a verdict for a test nobody ran: leave the case without one and tell the engineer what remains to run.
- Point to the page of the documentation in brackets for more.
- Name commands by their exact title as above. Never invent a command, a menu or a page.`;
}
