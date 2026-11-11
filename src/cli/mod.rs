pub mod debug;

pub mod diagram_service;
pub mod edit;
pub mod table; // tables hook
mod partition;
mod propose;
mod realize; // realization hook
mod schedule;
pub mod impact; // impact hook
pub mod semgit; // semgit hook
pub mod cicd; // cicd hook
pub mod review;
pub mod repl;
pub mod language_server;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(name = "arclang")]
#[clap(author = "ArcLang Contributors")]
#[clap(version)]
#[clap(about = "Industrial-grade Arcadia-as-Code compiler", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    
    #[clap(short, long, global = true)]
    pub verbose: bool,
    
    /// Accepted for the scripts that pass it: no command reads a
    /// configuration file, and the run says so.
    #[clap(short, long, global = true, hide = true)]
    pub config: Option<PathBuf>,

    /// The file the input stands for, when the input is a copy of it (an
    /// editor's buffer written to a temporary file): its imports, its
    /// cached verdicts and the files next to it are those of this file.
    #[clap(long = "as", global = true, value_name = "PATH")]
    pub origin: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum MetamodelFormat {
    Markdown,
    Json,
    /// A SysML v2 package using every standard-library element the SysML
    /// export can write (input of tools/sysml_library_index.py)
    #[clap(name = "sysml-library-probe")]
    SysmlLibraryProbe,
}

#[derive(Subcommand)]
pub enum Commands {
    Build {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
        
        /// Accepted for the scripts that pass it: every build compiles the
        /// whole model, and the run says so.
        #[clap(long, hide = true)]
        incremental: bool,
        
        #[clap(long)]
        release: bool,
        
        #[clap(long)]
        target: Option<String>,
    },
    
    Check {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        lint: bool,

        /// With --lint, the method whose rules are reported: arcadia (default) or cesam.
        /// A model declaring `method: cesam` is checked against CESAM without it.
        #[clap(long)]
        method: Option<String>,
        
        #[clap(long)]
        safety: bool,

        /// Print the findings as JSON, each named after its rule (see `arclang rules`)
        #[clap(long)]
        json: bool,

        /// text (default) or sarif: the diagnostics as a SARIF 2.1.0 log
        #[clap(long, value_enum)] // cicd hook
        format: Option<cicd::DiagnosticFormat>,
    },

    /// Arcadia transition: declare on the next level what realizes the elements of this one
    /// (oa-to-sa, sa-to-la, la-to-pa). Prints the edited model; --write writes it back.
    Transition {
        #[clap(value_parser)]
        input: PathBuf,
        /// oa-to-sa, sa-to-la or la-to-pa; omitted with --plan: every step
        #[clap(long)]
        step: Option<String>,
        /// The logical component (sa-to-la) or physical node (la-to-pa) that receives the created elements
        #[clap(long)]
        into: Option<String>,
        /// Only say what the step would create, as JSON
        #[clap(long)]
        plan: bool,
        #[clap(long)]
        write: bool,
    },

    /// Print the catalog of validation rules (what `check` reports) as Markdown or JSON
    Rules {
        #[clap(long, default_value = "markdown")]
        format: MetamodelFormat,
    },
    
    /// Format .arc sources: whitespace only, comments and order are kept
    #[clap(alias = "fmt")]
    Format {
        /// A .arc file, or a directory formatted recursively
        #[clap(value_parser)]
        input: PathBuf,

        /// Change nothing; exit 1 if any file is not formatted
        #[clap(long, conflicts_with = "write")]
        check: bool,

        /// Rewrite the files in place instead of printing to stdout
        #[clap(long)]
        write: bool,
    },
    
    /// Start a model from a template: `arclang new door --template cesam`
    /// writes door.arc, a model following CESAM with one section per vision,
    /// every element a placeholder to rename, which compiles and passes the gate
    New {
        #[clap(value_parser)]
        name: String,

        /// The template: cesam
        #[clap(long)]
        template: Option<String>,

        /// Write the file here instead of <name>.arc in the current directory
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
    },
    
    Sync {
        #[clap(subcommand)]
        sync_command: SyncCommands,
    },
    
    Plugin {
        #[clap(subcommand)]
        plugin_command: PluginCommands,
    },
    
    Trace {
        #[clap(value_parser)]
        input: PathBuf,

        /// With --review: only the traces from this element (id or name).
        #[clap(long)]
        from: Option<String>,

        /// With --review: only the traces to this element (id or name).
        #[clap(long)]
        to: Option<String>,

        #[clap(long)]
        validate: bool,

        /// List the suspect traces: reviewed, and one of their ends changed since.
        #[clap(long)]
        suspects: bool,

        /// Mark traces as reviewed (all, or those --from / --to): write the
        /// fingerprints of their two ends in the file, as `reviewed: "..."`.
        #[clap(long)]
        review: bool,

        /// With --review: print the reviewed source instead of writing the file.
        #[clap(long)]
        print: bool,

        #[clap(long)]
        matrix: bool,
    },

    /// Production-readiness gate: PASS/FAIL verdict against what an
    /// industrial design review requires (requirements coverage, HARA/ASIL
    /// consistency, timing budgets, ICD completeness)
    Gate {
        #[clap(value_parser)]
        input: PathBuf,

        #[clap(long, default_value = "iso26262")]
        standard: SafetyStandard,

        /// text (default) or sarif: the gate's findings as a SARIF 2.1.0 log
        #[clap(long, value_enum)] // cicd hook
        format: Option<cicd::DiagnosticFormat>,
    },

    /// Semantic diff between two model versions: compares by stable
    /// identity (UUID), so moving blocks is no change and renaming an
    /// element is a modification, not a remove+add
    Diff {
        /// The old version of the model (or, with --from, the model file)
        #[clap(value_parser)]
        old: PathBuf,

        /// The new version of the model (omit with --from)
        #[clap(value_parser)]
        new: Option<PathBuf>,

        /// Output as JSON (for CI pipelines)
        #[clap(long)]
        json: bool,

        /// Compare the file as committed at this git revision...
        #[clap(long)] // semgit hook
        from: Option<String>,

        /// ...with the file at this revision (default: the working tree, WORKTREE)
        #[clap(long)] // semgit hook
        to: Option<String>,

        /// text, json, md or html: the rich report with layers, consequences and layout
        #[clap(short, long, value_enum)] // semgit hook
        format: Option<semgit::DiffFormat>,

        /// Write the report to this file
        #[clap(short, long)] // semgit hook
        output: Option<PathBuf>,

        /// The standard of the gate in the consequences: ISO26262, DO178C or IEC61508
        #[clap(long, default_value = "ISO26262")] // semgit hook
        gate_standard: String,
    },

    /// The commits that changed one element, found by the semantic diff
    /// between successive revisions of the model
    History(semgit::HistoryArgs), // semgit hook

    /// Semantic three-way merge by element identity: a git merge driver for
    /// .arc and .layout.json files (`arclang merge --install`)
    Merge(semgit::MergeArgs), // semgit hook

    /// The model with elements (or the whole file) as they were at a git
    /// revision, printed for review: time travel, never a rewrite of history
    Restore(semgit::RestoreArgs), // semgit hook

    /// Baselines an auditor accepts: the gate, the coverage and the exports,
    /// bound to a git tag by a manifest
    Baseline {
        #[clap(subcommand)]
        command: semgit::BaselineCommand,
    }, // semgit hook

    /// CI/CD: write the pipelines (`ci init`), run what a pipeline runs (`ci run`)
    Ci {
        #[clap(subcommand)]
        command: cicd::CiCommand,
    }, // cicd hook

    /// The pull-request report: semantic diff, impact, gate before and after
    Report(cicd::ReportArgs), // cicd hook

    /// Compiler diagnostics and gate blockers as a SARIF 2.1.0 log
    Sarif(cicd::SarifArgs), // cicd hook

    /// Change-impact analysis: what a change to elements, a proposed edit
    /// (what-if) or the changes since a git revision reach, and why
    Impact(impact::ImpactArgs), // impact hook

    /// Run a state machine of the model step by step: events fired in order
    /// (--events), or a debugger for VS Code (--dap, the Debug Adapter Protocol)
    Debug(debug::DebugArgs),
    
    /// Verification by simulation: run the test cases that `simulate:` in
    /// OpenModelica (ARCLANG_OMC, or Docker with ARCLANG_OMC_DOCKER), check
    /// their `expects:` and cache the verdicts for `check`, `gate` and tables
    Verify {
        #[clap(value_parser, required_unless_present = "check_tools")]
        input: Option<PathBuf>,

        /// Do not simulate: report what is installed to run OpenModelica
        /// (Docker, the image, the Modelica library, a local omc); with
        /// --json, as a machine-readable diagnosis
        #[clap(long)]
        check_tools: bool,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// Print each outcome as JSON
        #[clap(long)]
        json: bool,

        /// Do not simulate: print, as JSON, the results kept of the last
        /// run of this test case (id or name), for the workbench to replay
        /// on the diagram: the time, each part's values, each expectation
        /// and the first instant it fails
        #[clap(long, value_name = "CASE")]
        replay: Option<String>,

        /// With --replay: the point of the sweep (0 is the first)
        #[clap(long, default_value_t = 0)]
        point: usize,

        /// Do not simulate: list, as JSON, the test cases that simulate and
        /// whether results of their last run are kept for --replay
        #[clap(long)]
        list: bool,

        /// Do not simulate: analyze the samples of the last run of a Monte
        /// Carlo case (--test) with R or Python (`r` or `python`): failure
        /// probability by a normal fit, influence of each parameter (PRCC,
        /// SRRC; delta and S1 with SALib). The verdict stays the compiler's
        #[clap(long, value_name = "ENGINE")]
        statistics: Option<String>,

        /// Do not simulate: list, as JSON, the Modelica classes of the model
        /// worth drawing (each simulated case's model, each configuration's
        /// system)
        #[clap(long)]
        drawings: bool,

        /// Do not simulate: print OpenModelica's instance of this Modelica
        /// class (getModelInstance: its components, icons, placements and
        /// connections) as JSON, for VS Code to draw it
        #[clap(long, value_name = "CLASS")]
        drawing: Option<String>,
    },

    /// Co-simulation in OMSimulator: each test case that `execute:`s, as a
    /// standard FMI co-simulation. The behavior's FMU (`export -f fmu`), the
    /// plant's (built by OpenModelica, or the FMU the case gives) and the
    /// case's inputs are connected in an SSP package that OMSimulator runs;
    /// the run is judged as SIL judges the C: the `expects:` on it, step by
    /// step against the model's own loop. Without OMSimulator here, the
    /// command runs in ArcLang Studio (Docker). See docs/cosimulation.md
    Cosim {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// The C compiler the behavior's FMU is built with. Default: $CC, else cc
        #[clap(long = "cc")]
        compiler: Option<String>,

        /// Keep each case's SSP (cosim.ssp, the FMUs inside), result and logs in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// Print the outcomes as JSON
        #[clap(long)]
        json: bool,
    },

    /// Software in the loop: compile the C generated from the state machines
    /// (`export -f c`) on this computer, run each test case that `execute:`s
    /// with its input vectors, compare the outputs with the model-level run
    /// step by step and judge its `expects:` on both
    Sil {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// The C compiler (gcc, clang, gcc-15...); repeat it to run every case
        /// with each. Default: $CC, else cc
        #[clap(long = "cc")]
        compilers: Vec<String>,

        /// Keep the generated files, harnesses and binaries in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// Print the outcomes as JSON
        #[clap(long)]
        json: bool,
    },

    /// Processor in the loop: build the generated C for a target package
    /// (Zephyr, `zephyr/litex_vexriscv`), run it on the target's processor in
    /// its emulator (Renode), drive each test case that `execute:`s through
    /// the bound channels (or the step() API), compare the outputs with the
    /// model-level run step by step and judge its `expects:` on both
    Pil {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// The target package (default: the io_binding's `target:`)
        #[clap(long)]
        target: Option<String>,

        /// Keep the applications, builds, plans and logs in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// Print the outcomes as JSON
        #[clap(long)]
        json: bool,
    },

    /// Hardware in simulation: the VHDL of each state machine allocated to
    /// programmable logic (`export -f vhdl`) simulated by GHDL through its
    /// registers, each test case that `execute:`s compared step by step with
    /// the bit-exact fixed-point C (difference 0) and with the model (within
    /// the quantisation tolerance), its `expects:` judged; `--synth` also
    /// synthesises it for its node's ECP5 (Yosys, nextpnr-ecp5, ecppack)
    Hdl {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// Synthesise, place and route each block, report its resources and Fmax
        #[clap(long)]
        synth: bool,

        /// With --synth: every machine as if allocated to this ECP5 part, at this clock (DEVICE[@MHZ], e.g. LFE5U-25F-6BG381C@25)
        #[clap(long, value_name = "DEVICE[@MHZ]")]
        as_if: Option<String>,

        /// With --synth: only this machine (name or id)
        #[clap(long)]
        machine: Option<String>,

        /// Keep the VHDL, the testbenches, the C reference and the netlists in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// Print the outcomes as JSON
        #[clap(long)]
        json: bool,
    },

    /// The processor and the programmable logic together: one LiteX SoC
    /// (a VexRiscv running the Zephyr application, the blocks of the
    /// machines allocated to programmable logic on its bus) simulated cycle
    /// by cycle in Verilator, each test case that `execute:`s such a machine
    /// run through the driver and compared, as `hdl` compares the VHDL, with
    /// the fixed-point C and the model; `--board` also builds the same SoC
    /// into one bitstream; `--port` runs the cases on the board instead
    Soc {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this test case (id or name)
        #[clap(long)]
        test: Option<String>,

        /// Build the bitstream, for the node's board (board:, else the board of its part)
        #[clap(long)]
        bitstream: bool,

        /// Build the bitstream for this board (ulx3s, colorlight_i5, colorlight_i9, ecp5_evn)
        #[clap(long)]
        board: Option<String>,

        /// Do not simulate (with --board: the bitstream only)
        #[clap(long)]
        no_sim: bool,

        /// Simulate the SoC as synthesised for the ECP5 by Yosys (a netlist of ECP5 cells), not as written
        #[clap(long)]
        gate_level: bool,

        /// Run the cases on the board through this serial port, the board holding the bitstream (implies --no-sim)
        #[clap(long)]
        port: Option<String>,

        /// Keep the application, the SoC, the simulator and the bitstream in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// Print the outcomes as JSON
        #[clap(long)]
        json: bool,
    },

    /// HW/SW partitioning: every machine measured as C on the processor
    /// (arclang pil: the step's time, the code's footprint) and as VHDL on
    /// the logic (arclang hdl --synth --as-if: resources, Fmax), what fits
    /// its period, what the model allocates today; kept beside the model
    /// for the table kind `partition`.
    Partition {
        #[clap(value_parser)]
        input: PathBuf,

        /// The target package of the C (else the bindings', else zephyr/litex_vexriscv)
        #[clap(long)]
        target: Option<String>,

        /// The ECP5 part of the logic (else the first logic node's, else LFE5U-25F-6BG381C)
        #[clap(long)]
        device: Option<String>,

        /// The clock of the logic, MHz (else the node's, else 25)
        #[clap(long)]
        clock: Option<f64>,

        /// Only this machine (name or id)
        #[clap(long)]
        machine: Option<String>,

        /// Print the two runs, run nothing
        #[clap(long)]
        dry_run: bool,

        /// The measurements as JSON
        #[clap(long)]
        json: bool,
    },

    /// Tasks and scheduling: the io_bindings of a target as tasks (a
    /// period and a priority each), the worst response time of each from
    /// the step times arclang partition measured, whether every task meets
    /// its period (fixed-priority response-time analysis).
    Schedule {
        #[clap(value_parser)]
        input: PathBuf,

        /// The target package (else the bindings')
        #[clap(long)]
        target: Option<String>,

        /// A step time given instead of measured: MACHINE=MICROSECONDS (repeatable)
        #[clap(long, value_name = "MACHINE=US")]
        wcet: Vec<String>,

        /// Run every task together on the target first (its emulator): each period proved kept, the step times under load
        #[clap(long)]
        prove: bool,

        /// With --prove: cycles of the longest period to run
        #[clap(long, default_value_t = 10)]
        cycles: usize,

        /// With --prove: keep the application, the build, the plan and the logs in this directory
        #[clap(long)]
        keep: Option<PathBuf>,

        /// The analysis as JSON
        #[clap(long)]
        json: bool,
    },

    /// Reverse engineering: the state machines a piece of code implements,
    /// as ArcLang. Reads the C the compiler generates (`arclang export -f
    /// c`): a directory holding `<model>.h` and `<model>.c`, or one of the
    /// two files. --check generates the C of the recovered model again and
    /// compares it with the C read: equal, the recovery is proven; any
    /// difference is printed and the command fails.
    Recover {
        #[clap(value_parser)]
        input: PathBuf,

        /// Write the recovered model to this file (printed otherwise)
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,

        /// Prove the recovery: the recovered model must generate the C it was read from
        #[clap(long)]
        check: bool,

        /// The programmable logic of the same model: the directory `arclang
        /// export -f vhdl` wrote. The machines it computes are sized from
        /// its register maps, and --check proves them too: the recovered
        /// model must generate every file of the logic as it was read
        #[clap(long, value_name = "DIR")]
        hdl: Option<PathBuf>,

        /// The input is code written by hand (an Arduino sketch): a proposer
        /// (--with) proposes a state machine and its correspondence, the
        /// proof of `arclang prove` judges, and what it refutes goes back to
        /// the proposer. The model is written only once it is proven
        #[clap(long)]
        propose: bool,

        /// With --propose: the command that proposes. It reads the prompt
        /// (the sketch included) on its input and prints its answer; any AI
        /// with a command line does. ARCLANG_PROPOSER names it otherwise
        #[clap(long, value_name = "COMMAND")]
        with: Option<String>,

        /// With --propose: how many answers the proposer may give
        #[clap(long, default_value_t = 4)]
        turns: usize,

        /// With --propose: the time one proof may take, in seconds
        #[clap(long, default_value_t = 300)]
        timeout: u64,

        /// With --propose: keep each answer and the files of its proof in this directory
        #[clap(long, value_name = "DIR")]
        keep: Option<PathBuf>,
    },

    /// Prove that code written by hand is a machine of the model, as each
    /// `correspondence` of the model says it is: after `setup()` and one
    /// `loop()` the code and the model correspond, and from any two states
    /// that correspond one `loop()` and one step leave two that correspond.
    /// The sources are read beside the model. CBMC and Bitwuzla decide (in
    /// ArcLang's formal image: tools/embedded/formal). Fails unless every
    /// correspondence is proven.
    Prove {
        #[clap(value_parser)]
        model: PathBuf,

        /// Only this correspondence
        #[clap(long, value_name = "NAME")]
        correspondence: Option<String>,

        /// The seconds the solver is given for each obligation (default 300)
        #[clap(long, value_name = "SECONDS", default_value_t = 300)]
        timeout: u64,

        /// Keep the files of the proof in this directory
        #[clap(long, value_name = "DIR")]
        keep: Option<PathBuf>,

        /// Print the result as JSON
        #[clap(long)]
        json: bool,
    },

    /// What the code as built implements against what the model says. The
    /// machines are read back from the C as built (as `recover` reads them)
    /// and from the C the model generates, and compared: each difference is
    /// said in the terms of the model, with its line in the code. Lines of
    /// the code that its machines do not explain are listed too. Fails when
    /// there is a difference.
    Drift {
        /// The model as designed
        #[clap(value_parser)]
        model: PathBuf,

        /// The C as built: a directory holding `<model>.h` and `<model>.c`, or one of the two
        #[clap(value_parser)]
        code: PathBuf,

        /// The VHDL as built too: the directory `arclang export -f vhdl` wrote
        /// (the VHDL, the register map, the driver, the fixed-point C), each
        /// file against the one the model generates
        #[clap(long, value_name = "DIR")]
        hdl: Option<PathBuf>,

        /// Print the result as JSON
        #[clap(long)]
        json: bool,

        /// Prove, or refute, that the C as built does what the model's C
        /// does: one step of each from any state and for any inputs, every
        /// field of the state and of the outputs compared (CBMC and
        /// Bitwuzla, in ArcLang's formal image: tools/embedded/formal).
        /// Two codes written differently that do the same are proven the
        /// same; two that do not come with the state and the inputs that
        /// tell them apart. With --hdl, each entity whose VHDL differs is
        /// proven too (GHDL, Yosys and SymbiYosys): from reset and for any
        /// inputs the two agree at every clock, or the clock and the
        /// outputs at which they differ are given
        #[clap(long)]
        prove: bool,

        /// With --prove: the seconds the solver is given (default 300)
        #[clap(long, value_name = "SECONDS", default_value_t = 300)]
        timeout: u64,
    },

    /// Climb the realization ladder (docs/realization.md): for each
    /// requirement, run in order what each stage is missing or stale
    /// (verify, sil, pil, hdl, soc, twin), read the ladder back after each
    /// step, stop a requirement's climb at a failure. --dry-run prints the
    /// plan and the gaps (a requirement nothing climbs) without running.
    Realize {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this requirement (id or name)
        #[clap(long)]
        requirement: Option<String>,

        /// Climb up to this stage only: simulation, sil, pil, hdl, soc, board or twin
        #[clap(long, value_name = "STAGE")]
        up_to: Option<String>,

        /// Print the plan and the gaps, run nothing
        #[clap(long)]
        dry_run: bool,

        /// Go on past a failure or an error (every step that can run runs)
        #[clap(long)]
        keep_going: bool,

        /// Include the board (the bitstream through --port): off by default, it needs the hardware
        #[clap(long)]
        board: bool,

        /// The plan, the runs and the ladder after, as JSON (the steps' output captured)
        #[clap(long)]
        json: bool,
    },

    /// Run the digital twins of a model: each `twin`'s physical model
    /// driven by its measured data (drives:) in OpenModelica, compared to
    /// the measurements (compares:) and checked against its expects:
    Twin {
        #[clap(value_parser)]
        input: PathBuf,

        /// Only this twin (id or name)
        #[clap(long)]
        twin: Option<String>,

        /// Print the outcomes as JSON (with the curves, for VS Code)
        #[clap(long)]
        json: bool,

        /// Do not run: list the twins as JSON
        #[clap(long)]
        list: bool,

        /// Follow the twin's live data (mqtt, opcua, kafka, http): run it over
        /// the latest window at every refresh, until stopped (or --for);
        /// with --json, one JSON line per run
        #[clap(long)]
        live: bool,

        /// With --live: stop after this many seconds
        #[clap(long = "for", value_name = "SECONDS")]
        for_seconds: Option<u64>,
    },

    /// A document filled from a template: Markdown whose {{ ... }}
    /// directives the compiler fills from the model (model, date, value,
    /// table, elements, verdicts, constraints, gate, suspects, rules).
    /// Without --template, a design review of the whole model. The output's
    /// extension chooses the format: .md, .html or .docx (Word)
    Document {
        #[clap(value_parser)]
        input: PathBuf,

        /// The template (Markdown with {{ ... }} directives)
        #[clap(long, short)]
        template: Option<PathBuf>,

        /// Where the document is written: .md, .html or .docx
        #[clap(long, short)]
        output: PathBuf,
    },

    Export {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
        
        #[clap(short, long)]
        format: ExportFormat,

        /// Modelica only: write the structure without the Placement, Line,
        /// Icon and Diagram annotations that make Modex and OMEdit draw it
        #[clap(long)]
        no_placement: bool,

        /// Modelica only: one file per class in a `<Package>/` folder (the
        /// structured layout of a Modelica library) instead of one file;
        /// Modex browses a package by its folders
        #[clap(long)]
        split: bool,

        /// Simulink connector only: what to export, as the Capella-Simulink
        /// connector's operations: functions, functional-chains,
        /// state-machines or data-pkg
        #[clap(long, default_value = "functions")]
        strategy: String,

        /// Simulink connector only: the elements to export, by Capella id or
        /// by name (repeat it, or separate them by commas); by default the
        /// strategy's roots (the top functions, every chain, every state
        /// machine, the layers' data packages)
        #[clap(long = "element", value_delimiter = ',')]
        elements: Vec<String>,

        /// Simulink connector only: leaf functions as `model` (one Simulink
        /// model each, referenced) or as `subsystem`
        #[clap(long, default_value = "model")]
        leaf_functions: String,

        /// Simulink connector only: ports named after the functional
        /// exchange (`ExchangeItem`) or after the Capella port (`CapellaElement`)
        #[clap(long, default_value = "ExchangeItem")]
        port_naming: String,

        /// Zephyr only: the target package (default: the io_binding's `target:`)
        #[clap(long)]
        target: Option<String>,

        /// Zephyr only: the io_binding the controller runs (default: the only one)
        #[clap(long)]
        binding: Option<String>,
    },
    
    Import {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long)]
        format: ImportFormat,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
    },
    
    Safety {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        standard: SafetyStandard,
        
        #[clap(long)]
        fmea: bool,
        
        /// The fault tree of every hazard: its failure modes, rate and budget
        #[clap(long)]
        fta: bool,

        /// A safety report in Markdown (hazards, FMEA by risk, fault trees, mitigations)
        #[clap(long)]
        report: bool,

        /// Where the report is written (default: the terminal)
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Start the HTTP API. Models given here are served through the
    /// Systems Modeling API under /api/systems-modeling/projects
    Serve {
        #[clap(long, default_value = "5001")]
        port: u16,

        /// Model files to serve (one project per model)
        #[clap(value_parser)]
        models: Vec<PathBuf>,

        /// Serve each model with its git history: one API commit per git
        /// commit that touched the file, the working tree as head
        #[clap(long)]
        history: bool,

        /// Number of git commits served per model with --history
        #[clap(long, default_value = "50")]
        history_depth: usize,

        /// Require a bearer token on every API request. The token is the
        /// content of this file; without the option it is read from the
        /// ARCLANG_API_TOKEN environment variable when that is set
        #[clap(long, value_parser)]
        token_file: Option<PathBuf>,

        /// Declare the API users: a file with one `name role token` per
        /// line, role being `read` or `write` and token given in clear or
        /// as `sha256:<hex digest>`. Every request must then present the
        /// token of one of them
        #[clap(long, value_parser)]
        users_file: Option<PathBuf>,

        /// Accept writes: POST .../commits changes the model file and makes
        /// one git commit per request. Needs a token, or a user who may write
        #[clap(long)]
        allow_write: bool,
    },
    
    Lsp {
        #[clap(long)]
        stdio: bool,
        
        #[clap(long)]
        port: Option<u16>,
    },
    
    Explorer {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
        
        #[clap(long, help = "Open in browser after generation")]
        open: bool,
    },
    
    Repl {
        #[clap(value_parser)]
        project: Option<PathBuf>,
    },
    
    Clean {
        #[clap(value_parser)]
        project: PathBuf,
        
        #[clap(long)]
        cache: bool,
    },
    
    Info {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(long)]
        metrics: bool,
        
        #[clap(long)]
        dependencies: bool,
    },
    
    /// Set one attribute of one element in a model file (comments and
    /// layout are kept; refused unless the edited model compiles)
    Set {
        /// The .arc file that declares the element
        #[clap(value_parser)]
        input: PathBuf,

        /// Identifier of the element, or its name when it writes no id
        element: String,

        /// Attribute name
        key: String,

        /// Value, as ArcLang source: '"text"', '25 ms', '[A, B]'
        #[clap(allow_hyphen_values = true)]
        value: String,

        /// Rewrite the file in place instead of printing to stdout
        #[clap(long)]
        write: bool,
    },

    /// Ask a judgment model (TypeSafe's Jev) whether the traces of a model
    /// are plausible. Advice only: probabilities, never a verdict. Sends
    /// the names and descriptions of the traced elements to api.typesafe.ai
    /// and needs TYPESAFE_API_KEY (environment or ./.env)
    Review {
        #[clap(value_parser)]
        input: PathBuf,

        /// Report declared traces under this plausibility
        #[clap(long, default_value = "0.3")]
        threshold: f64,

        /// Also judge undeclared component/function-to-requirement pairs
        /// and report those that may be a missing trace
        #[clap(long)]
        suggest: bool,

        /// Report undeclared pairs over this plausibility (with --suggest)
        #[clap(long, default_value = "0.8")]
        suggest_threshold: f64,

        /// Most questions asked; declared traces come first
        #[clap(long, default_value = "200")]
        max_questions: usize,

        /// Print the result as JSON
        #[clap(long)]
        json: bool,

        /// Say what would be sent, and send nothing
        #[clap(long)]
        dry_run: bool,
    },

    /// Rename one element in a model file. Refused when the element is
    /// known by its name elsewhere: its identity must not change
    Rename {
        #[clap(value_parser)]
        input: PathBuf,

        /// Identifier of the element
        element: String,

        /// The new name
        name: String,

        /// Rewrite the file in place instead of printing to stdout
        #[clap(long)]
        write: bool,
    },

    /// Remove one attribute of one element in a model file
    Unset {
        #[clap(value_parser)]
        input: PathBuf,

        /// Identifier of the element, or its name when it writes no id
        element: String,

        /// Attribute name
        key: String,

        /// Rewrite the file in place instead of printing to stdout
        #[clap(long)]
        write: bool,
    },

    /// Everything a modeling workbench shows of one model, as JSON: the
    /// diagrams as the viewer mounts them, the explorer document, every
    /// element with its typed attributes, the findings (named after their
    /// rule) and the production gate. A model that does not compile gives
    /// `{"success": false, "errors": [...]}`
    Workbench {
        #[clap(value_parser)]
        input: PathBuf,

        /// The standard the gate is run against: ISO26262, DO178C or IEC61508
        #[clap(long, default_value = "ISO26262")]
        gate_standard: String,

        /// Also render the interop exports (SysML v2, ReqIF, C headers, Protobuf)
        #[clap(long)]
        exports: bool,

        /// The model as committed at this git revision, with its layout file
        #[clap(long)] // semgit hook
        rev: Option<String>,
    },

    /// Apply one edit, written as JSON, to a model file: {"op": "set" |
    /// "unset" | "rename" | "add" | "add_raw" | "remove" | "move" |
    /// "add_trace" | "remove_trace" | "wrap_fragment" | "wrap_invariant" | "reconnect" |
    /// "duplicate" | "duplicate_many", ...}. Comments and layout are kept; refused unless the
    /// edited model compiles and shows what was asked. Prints {"success":
    /// true, "source": ...} or {"success": false, "error": ...}; --write
    /// writes the file back
    ///
    /// Reconnect: {"op": "reconnect", "kind": K, "from": A, "to": B, "name":
    /// N?, "new_from": C?, "new_to": D?} moves the source and/or the target of
    /// the relationship of kind K from A to B (named N when several join
    /// them); both, swapped, reverse it. K is a kind of the element graph:
    /// ComponentExchange, FunctionalExchange, OperationalExchange,
    /// PhysicalLink, PhysicalExchange, Transition, Message, Trace or
    /// Association (`arclang workbench` lists every relationship with its
    /// kind). Refused when a port goes the wrong way (a source leaves by an
    /// out or inout port, a target arrives by an in or inout port), when an
    /// exchange would join an element to itself, or when the compiler warns
    /// about the result (a trace whose ends break the trace rules of
    /// `arclang metamodel`).
    ///
    /// Duplicate: {"op": "duplicate", "element": E, "container": C?, "layer":
    /// L?} copies the declaration of E with what it holds, beside it or into
    /// C or the layer L; every id of the copy is made unique (LC-X becomes
    /// LC-X-2), an element without id gets a new name. {"op":
    /// "duplicate_many", "elements": [E, ...], "container": C?} copies
    /// several at once, with the exchanges, links and traces declared
    /// between two of them, joined to the copies.
    ///
    /// A move of a port ({"op": "move", "element": "LC-A.p", "container":
    /// "LC-B"}) takes the ends of its exchanges with it. A rename of a name
    /// no element has renames the relationship of that name.
    Edit {
        #[clap(value_parser)]
        input: PathBuf,

        /// The edit request as JSON, or `-` to read it from stdin
        #[clap(long, allow_hyphen_values = true)]
        request: String,

        /// Rewrite the file in place (the JSON answer is still printed)
        #[clap(long)]
        write: bool,
    },

    /// Live tables and matrices of a model (requirement coverage,
    /// traceability, allocation, N², ICD, elements), computed by the
    /// compiler: JSON with ids and UUIDs, CSV, TSV, Markdown, HTML or Excel
    Table(table::TableArgs), // tables hook

    /// Print the language metamodel (element kinds, typed attributes,
    /// enumerations, units, trace rules) as Markdown or JSON
    Metamodel {
        #[clap(long, default_value = "markdown")]
        format: MetamodelFormat,
    },

    Diagram {
        #[clap(value_parser)]
        input: PathBuf,
        
        #[clap(short, long, value_parser)]
        output: PathBuf,
        
        #[clap(short, long, default_value = "mermaid")]
        format: DiagramFormat,
        
        #[clap(long, default_value = "System Architecture")]
        title: String,

        /// Restrict the output to one kind of view: oab, sab, lab, pab, msm
        /// (mode and state machines), es (scenarios), cap, cdb, pbs, req
        /// (requirements) or par (parametric); or cesam: the views of the CESAM
        /// framework, titled and ordered by its grid
        #[clap(long)]
        view: Option<String>,

        /// Layout file (folds, open containers, manual placement) to show
        /// the diagrams with. Default: `<model>.layout.json` next to the
        /// model, when it exists
        #[clap(long, value_parser)]
        layout: Option<PathBuf>,

        /// Color theme of the HTML viewer (-f html): a preset
        /// (arclang-default, capella-classic, colorblind-safe,
        /// print-grayscale, high-contrast, monochrome-document) or an
        /// arclang-theme.json file. A file that breaks a color rule is refused
        #[clap(long)]
        theme: Option<String>,

        #[clap(long)]
        open: bool,
    },
}

#[derive(Subcommand)]
pub enum SyncCommands {
    Pull {
        #[clap(long)]
        plm: Option<String>,
        
        #[clap(long)]
        requirements: Option<String>,
        
        #[clap(long)]
        dry_run: bool,
    },
    
    Push {
        #[clap(long)]
        plm: Option<String>,
        
        #[clap(long)]
        requirements: Option<String>,
        
        #[clap(long)]
        dry_run: bool,
    },
    
    Status {
        #[clap(value_parser)]
        project: PathBuf,
    },
    
    Configure {
        #[clap(long)]
        plm_type: String,
        
        #[clap(long)]
        url: String,
        
        #[clap(long)]
        credentials: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum PluginCommands {
    List,
    
    Install {
        #[clap(value_parser)]
        name: String,
        
        #[clap(long)]
        version: Option<String>,
    },
    
    Uninstall {
        #[clap(value_parser)]
        name: String,
    },
    
    Info {
        #[clap(value_parser)]
        name: String,
    },
    
    Enable {
        #[clap(value_parser)]
        name: String,
    },
    
    Disable {
        #[clap(value_parser)]
        name: String,
    },
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ExportFormat {
    Capella,
    JSON,
    YAML,
    XML,
    Markdown,
    Mermaid,
    PlantUML,
    HTML,
    PDF,
    Terraform,
    SysML,
    /// The abstract syntax of the SysML v2 export, as the JSON records the
    /// API serves under /api/sysml-v2
    #[clap(name = "sysml-json")]
    SysMLJson,
    Simulink,
    /// What the Capella-Simulink connector exports: the pivot (`.mlcore`)
    /// and the MATLAB script that builds the Simulink project; the input is
    /// an ArcLang model or a `.capella` file
    #[clap(name = "simulink-connector")]
    SimulinkConnector,
    FMI,
    ReqIF,
    CHeaders,
    Proto,
    /// The located elements and physical links on a map (RFC 7946 GeoJSON)
    #[clap(name = "geojson")]
    GeoJSON,
    /// The product structure and the shapes for CAD (STEP AP242, ISO 10303-242)
    Step,
    /// A Modelica package of the physical architecture (OpenModelica):
    /// `output` is a DIRECTORY
    Modelica,
    /// Portable C99 of the state machines (`<model>.h/.c`, init/step), the
    /// I/O interface `arclang_io.h`, the glue of each `io_binding` and the
    /// host implementation of the I/O for SIL: `output` is a DIRECTORY
    #[clap(name = "c")]
    C,
    /// The generated C as an FMI 2.0 co-simulation FMU (`<model>.fmu`, with
    /// this computer's binary when $CC or cc compiles it) and the inputs
    /// and model-level reference of each executed case: `output` is a DIRECTORY
    #[clap(name = "fmu")]
    Fmu,
    /// A Zephyr application of the model for a target package (`--target`,
    /// else the io_binding's): the C, the glue, the target's arclang_io,
    /// drivers, devicetree overlay, build files and emulator: `output` is a DIRECTORY
    #[clap(name = "zephyr")]
    Zephyr,
    /// VHDL-2008 of the state machines allocated to programmable logic (one
    /// entity each and its Wishbone wrapper), the register map (JSON, LiteX
    /// csr.csv), the C driver and the bit-exact fixed-point C reference:
    /// `output` is a DIRECTORY
    #[clap(name = "vhdl")]
    Vhdl,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ImportFormat {
    Capella,
    /// SysML v2: a .sysml or .kerml text, a directory of them (one model),
    /// or a JSON listing of Systems Modeling API records
    #[clap(name = "sysml")]
    SysML,
    Mermaid,
    PlantUML,
    JSON,
    YAML,
    XML,
    DOORS,
    ReqIF,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum SafetyStandard {
    ISO26262,
    DO178C,
    IEC61508,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum DiagramFormat {
    /// Viewpoint diagram model (OAB/SAB/LAB/PAB) as JSON
    Viewpoints,
    /// ELK layout graphs of the viewpoint diagrams as JSON
    Elk,
    /// Self-contained HTML viewer of the viewpoint diagrams
    Html,
    Mermaid,
    PlantUML,
    Operational,
    Functional,
    Sequence,
    StateMachine,
    Component,
    Physical,
    Class,
    Tree,
    Capability,
    FunctionalChain,
    All,
}

pub struct CliRunner {
    verbose: bool,
    config_path: Option<PathBuf>,
}

impl CliRunner {
    pub fn new(cli: &Cli) -> Self {
        Self {
            verbose: cli.verbose,
            config_path: cli.config.clone(),
        }
    }
    
    pub fn run(&self, command: Commands) -> Result<(), CliError> {
        if let Some(path) = &self.config_path {
            eprintln!("note: --config {} has no effect: no command reads a configuration file", path.display());
        }
        match command {
            Commands::Build { input, output, incremental, release, target } => {
                if incremental {
                    eprintln!("note: --incremental has no effect: a build compiles the whole model");
                }
                self.run_build(input, output, incremental, release, target)
            }
            Commands::Check { input, format: Some(cicd::DiagnosticFormat::Sarif), .. } => { // cicd hook
                cicd::run_scoped_sarif(&input, crate::compiler::cicd::diag::Scope::Check, &SafetyStandard::ISO26262)
            }
            Commands::Gate { input, standard, format: Some(cicd::DiagnosticFormat::Sarif) } => { // cicd hook
                cicd::run_scoped_sarif(&input, crate::compiler::cicd::diag::Scope::Gate, &standard)
            }
            Commands::Ci { command } => cicd::run_ci(&command), // cicd hook
            Commands::Report(args) => cicd::run_report(&args), // cicd hook
            Commands::Sarif(args) => cicd::run_sarif(&args), // cicd hook
            Commands::Check { input, lint, method, safety, json, format: _ } => {
                let cesam = match method.as_deref().map(str::to_ascii_lowercase).as_deref() {
                    None | Some("arcadia") => false,
                    Some("cesam") => true,
                    Some(other) => return Err(CliError::Compilation(format!("unknown method '{other}': arcadia or cesam"))),
                };
                if json {
                    self.run_check_json(input, cesam)
                } else {
                    self.run_check(input, lint, safety, cesam)
                }
            }
            Commands::Transition { input, step, into, plan, write } => {
                self.run_transition(input, step.as_deref(), into.as_deref(), plan, write)
            }
            Commands::Rules { format } => {
                match format {
                    MetamodelFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(crate::compiler::rules::catalog()).unwrap_or_default()
                    ),
                    _ => print!("{}", crate::compiler::rules::to_markdown()),
                }
                Ok(())
            }
            Commands::Format { input, check, write } => {
                self.run_format(input, check, write)
            }
            Commands::New { name, template, output } => {
                self.run_new(name, template, output)
            }
            Commands::Sync { sync_command } => {
                self.run_sync(sync_command)
            }
            Commands::Plugin { plugin_command } => {
                self.run_plugin(plugin_command)
            }
            Commands::Trace { input, from, to, validate, matrix, suspects, review, print } => {
                if review {
                    return self.run_trace_review(&input, from.as_deref(), to.as_deref(), print);
                }
                if suspects {
                    return self.run_trace_suspects(&input);
                }
                self.run_trace(input, from, to, validate, matrix)
            }
            Commands::Impact(args) => impact::run(&args), // impact hook
            Commands::Debug(args) => debug::run(&args),
            Commands::Diff { old, new, json, from, to, format, output, gate_standard } => {
                if from.is_some() || format.is_some() || output.is_some() { // semgit hook
                    let format = format.map(Into::into).unwrap_or(if json { crate::compiler::semgit::render::Format::Json } else { crate::compiler::semgit::render::Format::Text });
                    semgit::run_diff(old, new, from, to, format, output, &gate_standard)
                } else {
                    let new = new.ok_or_else(|| CliError::Config("give two model files, or one with --from <revision>".into()))?;
                    self.run_diff(old, new, json)
                }
            }
            Commands::History(args) => semgit::run_history(&args), // semgit hook
            Commands::Merge(args) => semgit::run_merge(&args), // semgit hook
            Commands::Restore(args) => semgit::run_restore(&args), // semgit hook
            Commands::Baseline { command } => semgit::run_baseline(&command), // semgit hook
            Commands::Gate { input, standard, format: _ } => {
                self.run_gate(input, standard)
            }
            Commands::Export { input, output, format, no_placement, split, strategy, elements, leaf_functions, port_naming, target, binding } => {
                if matches!(format, ExportFormat::Zephyr) {
                    return self.run_export_zephyr(&input, &output, target.as_deref(), binding.as_deref());
                }
                if target.is_some() || binding.is_some() {
                    return Err(CliError::Compilation("--target and --binding apply to the Zephyr export (-f zephyr)".to_string()));
                }
                if (no_placement || split) && !matches!(format, ExportFormat::Modelica) {
                    return Err(CliError::Compilation("--no-placement and --split apply to the Modelica export (-f modelica)".to_string()));
                }
                if matches!(format, ExportFormat::SimulinkConnector) {
                    let options = crate::compiler::simulink_connector::m2m::Options { leaf_functions, port_naming, draw_without_diagram: true, ..Default::default() };
                    return self.run_simulink_connector(&input, &output, &strategy, &elements, &options);
                }
                if !elements.is_empty() || strategy != "functions" || leaf_functions != "model" || port_naming != "ExchangeItem" {
                    return Err(CliError::Compilation("--strategy, --element, --leaf-functions and --port-naming apply to the Simulink connector export (-f simulink-connector)".to_string()));
                }
                self.run_export(input, output, format, no_placement, split)
            }
            Commands::Verify { input, test, json, check_tools, replay, point, list, statistics, drawings, drawing } => {
                if check_tools {
                    return self.run_check_tools(json);
                }
                let input = input.ok_or_else(|| CliError::Config("give a model file to verify".into()))?;
                if let Some(engine) = statistics {
                    return self.run_statistics(&input, test.as_deref(), &engine, json);
                }
                if drawings || drawing.is_some() {
                    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| CliError::Compilation(e.to_string()))?;
                    use crate::compiler::simulation::drawing;
                    let text = match drawing {
                        Some(class) => drawing::instance(&result, &input, &class, &crate::compiler::simulation::tool::ToolConfig::from_env()).map_err(CliError::Compilation)?,
                        None => drawing::drawables_json(&result).map_err(CliError::Compilation)?.to_string(),
                    };
                    println!("{text}");
                    return Ok(());
                }
                if list {
                    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| CliError::Compilation(e.to_string()))?;
                    let kept = crate::compiler::simulation::replay::dir(&input);
                    let cases: Vec<serde_json::Value> = result
                        .ast
                        .test_cases
                        .iter()
                        .filter(|t| crate::compiler::simulation::is_simulated(t))
                        .map(|t| serde_json::json!({ "id": t.id, "name": t.name, "replayable": crate::compiler::simulation::replay::kept(&kept, &t.id) }))
                        .collect();
                    println!("{}", serde_json::Value::Array(cases));
                    return Ok(());
                }
                if let Some(case) = replay {
                    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| CliError::Compilation(e.to_string()))?;
                    let replay = crate::compiler::simulation::run::replay(&result, &input, &case, point).map_err(CliError::Compilation)?;
                    println!("{}", serde_json::to_string(&replay).map_err(|e| CliError::Compilation(e.to_string()))?);
                    return Ok(());
                }
                self.run_verify(input, test, json)
            }
            Commands::Sil { input, test, compilers, keep, json } => self.run_sil(&input, test, compilers, keep, json),
            Commands::Cosim { input, test, compiler, keep, json } => self.run_cosim(&input, test, compiler, keep, json),
            Commands::Pil { input, test, target, keep, json } => self.run_pil(&input, test, target, keep, json),
            Commands::Hdl { input, test, synth, as_if, machine, keep, json } => self.run_hdl(&input, test, synth, as_if, machine, keep, json),
            Commands::Partition { input, target, device, clock, machine, dry_run, json } => partition::run(&input, partition::Options { target, device, clock_mhz: clock, machine, dry_run, json }),
            Commands::Schedule { input, target, wcet, prove, cycles, keep, json } => schedule::run(&input, schedule::Options { target, wcet, prove, cycles, keep, json }),
            Commands::Soc { input, test, bitstream, board, no_sim, gate_level, port, keep, json } => self.run_soc(&input, test, bitstream || board.is_some(), board, !no_sim && port.is_none(), gate_level, port, keep, json),
            Commands::Prove { model, correspondence, timeout, keep, json } => self.run_prove(&model, correspondence.as_deref(), timeout, keep, json),
            Commands::Recover { input, output, propose: true, with, turns, timeout, keep, .. } => propose::run(&input, propose::Options { with, turns, timeout, output, keep }),
            Commands::Recover { input, output, check, hdl, .. } => self.run_recover(&input, output.as_deref(), check, hdl.as_deref()),
            Commands::Drift { model, code, hdl, json, prove, timeout } => self.run_drift(&model, &code, hdl.as_deref(), json, prove.then_some(timeout)),
            Commands::Realize { input, requirement, up_to, dry_run, keep_going, board, json } => realize::run(&input, realize::Options { requirement, up_to, dry_run, keep_going, board, json }),
            Commands::Twin { input, twin, json, list, live, for_seconds } => {
                if live {
                    self.run_twin_live(input, twin, json, for_seconds)
                } else {
                    self.run_twin(input, twin, json, list)
                }
            }
            Commands::Document { input, template, output } => self.run_document(input, template, output),
            Commands::Import { input, format, output } => {
                self.run_import(input, format, output)
            }
            Commands::Safety { input, standard, fmea, fta, report, output } => {
                self.run_safety(input, standard, fmea, fta, report, output)
            }
            Commands::Serve { port, models, history, history_depth, token_file, users_file, allow_write } => {
                self.run_serve(port, models, history.then_some(history_depth), token_file, users_file, allow_write)
            }
            Commands::Lsp { stdio, port } => {
                self.run_lsp(stdio, port)
            }
            Commands::Explorer { input, output, open } => {
                self.run_explorer(input, output, open)
            }
            Commands::Repl { project } => {
                self.run_repl(project)
            }
            Commands::Clean { project, cache } => {
                self.run_clean(project, cache)
            }
            Commands::Info { input, metrics, dependencies } => {
                self.run_info(input, metrics, dependencies)
            }
            Commands::Diagram {
                input,
                output,
                format: format @ DiagramFormat::Html,
                view,
                layout,
                theme,
                ..
            } => self.run_viewpoint_diagrams(input, output, view, format, layout, theme),
            Commands::Diagram { theme: Some(_), format, .. } => Err(CliError::Compilation(format!(
                "--theme colors the HTML viewer (-f html), not {format:?}"
            ))),
            Commands::Diagram {
                input,
                output,
                format: format @ (DiagramFormat::Viewpoints | DiagramFormat::Elk),
                view,
                layout,
                ..
            } => self.run_viewpoint_diagrams(input, output, view, format, layout, None),
            Commands::Diagram { layout: Some(_), format, .. } => Err(CliError::Compilation(format!(
                "--layout applies to the viewpoint diagrams (-f html, viewpoints or elk), not to {format:?}"
            ))),
            Commands::Diagram { input, output, format, title, open, view: _, layout: None, theme: None } => {
                self.run_diagram(input, output, format, title, open)
            }
            Commands::Set { input, element, key, value, write } => {
                edit::run(&input, &element, &key, Some(&value), write)
            }
            Commands::Unset { input, element, key, write } => {
                edit::run(&input, &element, &key, None, write)
            }
            Commands::Rename { input, element, name, write } => edit::rename(&input, &element, &name, write),
            Commands::Review { input, threshold, suggest, suggest_threshold, max_questions, json, dry_run } => {
                let in_range = |value: f64| (0.0..=1.0).contains(&value);
                if !in_range(threshold) || !in_range(suggest_threshold) {
                    return Err(CliError::Config("a plausibility threshold is between 0 and 1".to_string()));
                }
                let options = crate::review::Options {
                    threshold,
                    suggest_over: suggest.then_some(suggest_threshold),
                    max_questions,
                };
                review::run(&input, options, json, dry_run)
            }
            Commands::Workbench { input, gate_standard, exports, rev } => match rev { // semgit hook
                Some(rev) => semgit::run_workbench_at(&input, &rev, &gate_standard, exports),
                None => edit::workbench(&input, &gate_standard, exports),
            },
            Commands::Edit { input, request, write } => edit::apply_request(&input, &request, write),
            Commands::Table(args) => table::run(&args), // tables hook
            Commands::Metamodel { format } => {
                let metamodel = crate::compiler::metamodel::Metamodel::current();
                match format {
                    MetamodelFormat::Markdown => print!("{}", metamodel.to_markdown()),
                    MetamodelFormat::Json => println!("{}", metamodel.to_json()),
                    MetamodelFormat::SysmlLibraryProbe => {
                        print!("{}", crate::compiler::sysmlv2_generator::library_probe())
                    }
                }
                Ok(())
            }
        }
    }
    
    fn run_build(
        &self,
        input: PathBuf,
        output: Option<PathBuf>,
        _incremental: bool,
        release: bool,
        target: Option<String>,
    ) -> Result<(), CliError> {
        // The file written holds what its extension says: the JSON model by
        // default, ArcLang's XML, or a Capella project (what `export -f
        // capella` writes: the .capella with its .aird and .afm).
        let (target, extension) = match target.as_deref() {
            None | Some("json") => ("json", "json"),
            Some("xml") => ("xml", "xml"),
            Some("capella") => {
                let output = output.unwrap_or_else(|| input.with_extension("capella"));
                return self.run_export(input, output, ExportFormat::Capella, false, false);
            }
            Some(other) => {
                return Err(CliError::Config(format!(
                    "unknown build target '{other}': expected json, xml or capella (other formats: arclang export)"
                )))
            }
        };

        println!("Building {}...", input.display());

        let config = crate::CompilerConfig {
            optimization_level: if release { 3 } else { 0 },
            target: target.to_string(),
            ..crate::CompilerConfig::default()
        };

        let mut compiler = crate::Compiler::new(config);

        match compiler.compile_file(&input) {
            Ok(result) => {
                let output_path = output.unwrap_or_else(|| input.with_extension(extension));
                let content = if target == "json" {
                    result
                        .ast
                        .to_json()
                        .map_err(|e| CliError::Compilation(format!("JSON export failed: {}", e)))?
                } else {
                    result.output.clone()
                };

                if let Err(e) = std::fs::write(&output_path, content) {
                    return Err(CliError::Io(e));
                }

                if !result.warnings.is_empty() {
                    eprintln!("⚠ {} warning(s):", result.warnings.len());
                    for warning in &result.warnings {
                        eprintln!("  warning: {}", warning);
                    }
                }

                println!("✓ Compilation successful");
                println!("  Output: {}", output_path.display());
                println!("  Requirements: {}", result.semantic_model.requirements.len());
                println!("  Components: {}", result.semantic_model.components.len());
                println!("  Functions: {}", result.semantic_model.functions.len());
                println!("  Traces: {}", result.semantic_model.traces.len());

                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Compilation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn print_diff(report: &crate::compiler::semantic_diff::DiffReport) {
        let print_changes = |changes: &[crate::compiler::semantic_diff::FieldChange]| {
            for change in changes {
                println!("      {}: \"{}\" -> \"{}\"", change.field, change.old, change.new);
            }
        };
        let relationship = |r: &crate::compiler::semantic_diff::RelationshipRef| {
            let name = if r.name.is_empty() { String::new() } else { format!(" '{}'", r.name) };
            format!("{}{} {} -> {}", r.kind, name, r.source, r.target)
        };

        for entry in &report.added {
            println!("  + added    {} '{}' [{}]", entry.element_type, entry.name, entry.id);
        }
        for entry in &report.removed {
            println!("  - removed  {} '{}' [{}]", entry.element_type, entry.name, entry.id);
        }
        for entry in &report.modified {
            println!(
                "  ~ modified {} '{}' [{}]",
                entry.element.element_type, entry.element.name, entry.element.id
            );
            print_changes(&entry.changes);
        }
        for trace in &report.traces_added {
            println!("  + trace    {} {} {}", trace.from, trace.trace_type, trace.to);
        }
        for trace in &report.traces_removed {
            println!("  - trace    {} {} {}", trace.from, trace.trace_type, trace.to);
        }
        for entry in &report.traces_modified {
            println!("  ~ trace    {} {} {}", entry.trace.from, entry.trace.trace_type, entry.trace.to);
            print_changes(&entry.changes);
        }
        for entry in &report.relationships_added {
            println!("  + relation {}", relationship(entry));
        }
        for entry in &report.relationships_removed {
            println!("  - relation {}", relationship(entry));
        }
        for entry in &report.relationships_modified {
            println!("  ~ relation {}", relationship(&entry.relationship));
            print_changes(&entry.changes);
        }
        println!(
            "\n  Total: {} added, {} removed, {} modified, {} trace(s) added, {} trace(s) removed, {} trace(s) modified, {} relationship change(s)",
            report.added.len(),
            report.removed.len(),
            report.modified.len(),
            report.traces_added.len(),
            report.traces_removed.len(),
            report.traces_modified.len(),
            report.relationships_added.len() + report.relationships_removed.len() + report.relationships_modified.len()
        );
    }

    fn run_transition(&self, input: PathBuf, step: Option<&str>, into: Option<&str>, plan_only: bool, write: bool) -> Result<(), CliError> {
        use crate::compiler::transition::{self, Step};
        let source = std::fs::read_to_string(&input)?;
        let step = match step {
            Some(text) => Some(Step::parse(text).ok_or_else(|| CliError::Compilation(format!("unknown step '{text}': use oa-to-sa, sa-to-la or la-to-pa")))?),
            None => None,
        };
        if plan_only || step.is_none() {
            let compiled = crate::Compiler::new(crate::CompilerConfig::default())
                .compile_string(&source)
                .map_err(|e| CliError::Compilation(e.to_string()))?;
            let steps: Vec<Step> = step.map(|s| vec![s]).unwrap_or_else(|| Step::ALL.to_vec());
            let plans: Vec<transition::Plan> = steps.into_iter().map(|s| transition::plan(s, &compiled.ast)).collect();
            println!("{}", serde_json::to_string_pretty(&plans).unwrap_or_default());
            return Ok(());
        }
        let step = step.expect("a step");
        let done = transition::apply(step, &source, into).map_err(CliError::Compilation)?;
        for created in &done.created {
            eprintln!("+ {} '{}' ({}) realizes {} in {}", created.kind, created.name, created.id, created.realizes, created.inside);
            if let Some(note) = &created.note {
                eprintln!("  note: {note}");
            }
        }
        if done.container_created {
            eprintln!("+ container '{}' created", done.container);
        }
        if done.created.is_empty() {
            eprintln!("{}: nothing to do, every element is realized", step.id());
        }
        if !write {
            print!("{}", done.source);
            return Ok(());
        }
        if done.source != source {
            std::fs::write(&input, &done.source)?;
            eprintln!("{} written", input.display());
        }
        Ok(())
    }

    /// `check --json`: the outcome as one JSON document — errors, or the
    /// findings each named after its rule, with the element concerned and
    /// the edit that answers it when one can be named.
    fn run_check_json(&self, input: PathBuf, cesam: bool) -> Result<(), CliError> {
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let document = match compiler.compile_file(&input) {
            Ok(result) => {
                let mut findings = crate::compiler::rules::findings(&result);
                if cesam && !crate::compiler::cesam_rules::declared(&result.ast) {
                    findings.extend(crate::compiler::rules::cesam_findings(&result));
                }
                let realization = crate::compiler::realization::ladders(&result, Some(&input));
                serde_json::json!({
                    "success": true,
                    "file": input.display().to_string(),
                    "errors": [],
                    "findings": findings,
                    "realization": realization,
                    "metrics": {
                        "requirements": result.semantic_model.requirements.len(),
                        "components": result.semantic_model.components.len(),
                        "functions": result.semantic_model.functions.len(),
                        "traces": result.semantic_model.traces.len(),
                    },
                })
            }
            Err(error) => serde_json::json!({
                "success": false,
                "file": input.display().to_string(),
                "errors": [error.to_string()],
                "findings": [],
            }),
        };
        println!("{}", serde_json::to_string_pretty(&document).unwrap_or_default());
        if document["success"] == true {
            Ok(())
        } else {
            Err(CliError::Compilation("check failed".to_string()))
        }
    }

    fn run_check(&self, input: PathBuf, lint: bool, safety: bool, cesam: bool) -> Result<(), CliError> {
        println!("Checking {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                println!("✓ No compilation errors");

                if !result.warnings.is_empty() {
                    println!("\n⚠ Compilation warnings:");
                    for warning in &result.warnings {
                        println!("  {}", warning);
                    }
                }

                if !result.semantic_model.constraints.is_empty() {
                    println!("\nConstraints:");
                    for constraint in &result.semantic_model.constraints {
                        println!(
                            "  {} {} — {}  [{} vs {}]",
                            if constraint.satisfied { "✓" } else { "✗" },
                            constraint.name,
                            constraint.expression,
                            constraint.left,
                            constraint.right
                        );
                    }
                }

                let warnings = result.semantic_model.validate_traceability();
                if !warnings.is_empty() {
                    println!("\n⚠ Traceability warnings:");
                    for warning in &warnings {
                        println!("  {}", warning);
                    }
                }

                let ladders = crate::compiler::realization::ladders(&result, Some(&input));
                if crate::compiler::realization::applies(&ladders) {
                    println!("\nRealization (what each stage says of each requirement; {}):", crate::compiler::realization::LEGEND);
                    for ladder in ladders.iter().filter(|l| l.applies()) {
                        println!("  {}", crate::compiler::realization::line(ladder));
                        if !ladder.next.is_empty() {
                            println!("      next: {}", ladder.next);
                        }
                    }
                }
                
                if lint {
                    let lints = crate::compiler::semantic::arcadia_methodology_lints(&result.ast);
                    if lints.is_empty() {
                        println!("\n✓ Arcadia methodology: no advisories");
                    } else {
                        println!("\nℹ Arcadia methodology advisories:");
                        for advisory in &lints {
                            println!("  {}", advisory);
                        }
                    }
                }

                if (lint && cesam) || crate::compiler::cesam_rules::declared(&result.ast) {
                    let advisories = crate::compiler::cesam_rules::advisories(&result.ast, &result.semantic_model);
                    if advisories.is_empty() {
                        println!("\n✓ CESAM methodology: no advisories");
                    } else {
                        println!("\nℹ CESAM methodology advisories ({}):", advisories.len());
                        for line in crate::compiler::cesam_rules::lines(&advisories) {
                            println!("  {}", line);
                        }
                    }
                }

                if lint || safety {
                    println!("\nModel metrics:");
                    let metrics = result.semantic_model.compute_metrics();
                    println!("  Total elements: {}", metrics.total_elements);
                    println!("  Requirements: {}", metrics.requirements_count);
                    println!("  Components: {}", metrics.components_count);
                    println!("  Traces: {}", metrics.traces_count);
                    println!("  Traceability: {:.1}%", metrics.traceability_coverage);
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Check failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_format(&self, input: PathBuf, check: bool, write: bool) -> Result<(), CliError> {
        let is_directory = input.is_dir();
        if is_directory && !check && !write {
            return Err(CliError::Config(
                "formatting a directory needs --write or --check".to_string(),
            ));
        }

        let mut files = Vec::new();
        collect_arc_files(&input, &mut files)?;
        files.sort();
        if files.is_empty() {
            return Err(CliError::Config(format!("no .arc file in {}", input.display())));
        }

        let mut unformatted = 0;
        let mut failed = 0;
        for path in &files {
            let source = std::fs::read_to_string(path)?;
            let formatted = match crate::compiler::format::format_source(&source) {
                Ok(formatted) => formatted,
                // One unreadable file must not hide the state of the others.
                Err(e) => {
                    eprintln!("✗ {}: {e}", path.display());
                    failed += 1;
                    continue;
                }
            };
            let changed = formatted != source;
            if changed {
                unformatted += 1;
            }

            if check {
                if changed {
                    println!("{}", path.display());
                }
            } else if write {
                if changed {
                    std::fs::write(path, &formatted)?;
                    println!("formatted {}", path.display());
                }
            } else {
                print!("{formatted}");
            }
        }

        if failed > 0 {
            return Err(CliError::Compilation(format!(
                "{failed} file(s) could not be formatted"
            )));
        }
        if check && unformatted > 0 {
            return Err(CliError::Unformatted(unformatted));
        }
        if write {
            println!("{} file(s) checked, {} rewritten", files.len(), unformatted);
        }
        Ok(())
    }

    /// The templates `arclang new` starts from, by name.
    const TEMPLATES: &'static [(&'static str, &'static str)] = &[("cesam", include_str!("templates/cesam.arc"))];

    fn run_new(&self, name: String, template: Option<String>, output: Option<PathBuf>) -> Result<(), CliError> {
        let available = || Self::TEMPLATES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ");
        let Some(template) = template else {
            return Err(CliError::Config(format!("arclang new needs --template <name>: one of {}", available())));
        };
        let Some((_, text)) = Self::TEMPLATES.iter().find(|(n, _)| n.eq_ignore_ascii_case(&template)) else {
            return Err(CliError::Config(format!("unknown template '{template}': one of {}", available())));
        };
        let identifier: String = name.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
        if identifier.is_empty() || !identifier.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') {
            return Err(CliError::Config(format!("'{name}' is not a model name: letters, digits and underscores, starting with a letter")));
        }
        let output = output.unwrap_or_else(|| PathBuf::from(format!("{identifier}.arc")));
        if output.exists() {
            return Err(CliError::Config(format!("{} exists already; choose another name or --output", output.display())));
        }
        let file = output.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        let source = text.replace("{{NAME}}", &identifier).replace("{{FILE}}", &file);
        // The template is checked before it is written: a model that does not compile is never left behind.
        crate::Compiler::new(crate::CompilerConfig::default())
            .compile_string(&source)
            .map_err(|e| CliError::Compilation(format!("the {template} template does not compile as {identifier}: {e}")))?;
        std::fs::write(&output, &source).map_err(|e| CliError::Config(format!("cannot write {}: {e}", output.display())))?;
        println!("✓ {} written from the {template} template", output.display());
        println!("  arclang check {} --lint", output.display());
        println!("  arclang table {} --kind cesam_grid -f md", output.display());
        Ok(())
    }

    fn run_sync(&self, _sync_command: SyncCommands) -> Result<(), CliError> {
        Err(CliError::NotImplemented(
            "PLM synchronization is not implemented yet".to_string(),
        ))
    }
    
    fn run_plugin(&self, _plugin_command: PluginCommands) -> Result<(), CliError> {
        Err(CliError::NotImplemented(
            "the plugin system is not implemented yet".to_string(),
        ))
    }
    
    fn run_document(&self, input: PathBuf, template: Option<PathBuf>, output: PathBuf) -> Result<(), CliError> {
        use crate::compiler::document;
        let fail = |e: String| CliError::Compilation(e);
        let text = match &template {
            Some(path) => std::fs::read_to_string(path).map_err(|e| fail(format!("cannot read the template {}: {e}", path.display())))?,
            None => document::DEFAULT_TEMPLATE.to_string(),
        };
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| fail(e.to_string()))?;
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let markdown = document::render(&text, &result, &date).map_err(|e| fail(format!("{}: {e}", template.as_deref().map(|p| p.display().to_string()).unwrap_or_else(|| "the default template".to_string()))))?;
        let title = result.semantic_model.name.clone().unwrap_or_else(|| "Model".to_string());
        let bytes = match output.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("md" | "markdown") => markdown.into_bytes(),
            Some("html" | "htm") => document::to_html(&markdown, &title).into_bytes(),
            Some("docx") => document::to_docx(&markdown, &title),
            _ => return Err(CliError::Config(format!("{}: the output is .md, .html or .docx", output.display()))),
        };
        std::fs::write(&output, bytes).map_err(|e| fail(format!("cannot write {}: {e}", output.display())))?;
        println!("Wrote {}", output.display());
        Ok(())
    }

    fn run_trace_suspects(&self, input: &std::path::Path) -> Result<(), CliError> {
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let graph = crate::compiler::elements::build(&result.ast, &result.semantic_model);
        let watched = result.ast.traces.iter().filter(|t| t.attributes.contains_key("reviewed")).count();
        let suspects = crate::compiler::suspect::suspects(&result.ast, &graph).map_err(CliError::Compilation)?;
        let verifications: usize = result.ast.test_cases.iter().map(|c| c.verifies.len()).sum();
        let never = suspects.iter().filter(|s| s.link == crate::compiler::suspect::Link::Verification && s.changed.is_empty()).count();
        let verified: usize = result.ast.test_cases.iter().filter(|c| c.attributes.contains_key("reviewed")).map(|c| c.verifies.len()).sum::<usize>().saturating_sub(never);
        let and_verifications = if verifications > 0 { format!(" and {verified} of {verifications} verification(s)") } else { String::new() };
        println!("{watched} of {} trace(s){and_verifications} reviewed, {} suspect", result.ast.traces.len(), suspects.len());
        for suspect in &suspects {
            println!("  {}", suspect.line().replace("<model>", &input.display().to_string()));
        }
        Ok(())
    }

    fn run_trace_review(&self, input: &std::path::Path, from: Option<&str>, to: Option<&str>, print: bool) -> Result<(), CliError> {
        use crate::compiler::{elements, source_edit, suspect};
        let fail = |e: String| CliError::Compilation(e);
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| fail(e.to_string()))?;
        let graph = elements::build(&result.ast, &result.semantic_model);
        // An end given by id or name matches the trace however it writes that end.
        let names = |written: &str| -> Vec<String> {
            match suspect::end(&graph, written) {
                Some(e) => vec![e.id.clone(), e.name.clone()],
                None => vec![written.to_string()],
            }
        };
        let wanted = |end: &str, filter: Option<&str>| filter.is_none_or(|f| names(f).iter().any(|n| n == end) || f == end);
        let traces: Vec<_> = result.ast.traces.iter().filter(|t| wanted(&t.from, from) && wanted(&t.to, to)).collect();
        // A test case's `verifies:` is reviewed like a trace from the case to the requirement.
        let cases: Vec<(&crate::compiler::ast::TestCase, Vec<&str>)> = result
            .ast
            .test_cases
            .iter()
            .filter(|c| wanted(&c.id, from) || wanted(&c.name, from))
            .map(|c| (c, c.verifies.iter().map(String::as_str).filter(|r| wanted(r, to)).collect::<Vec<_>>()))
            .filter(|(_, requirements)| !requirements.is_empty())
            .collect();
        if traces.is_empty() && cases.is_empty() {
            return Err(fail("no trace or verification matches: give --from and --to as the ids or names of its ends".to_string()));
        }
        let mut text = std::fs::read_to_string(input).map_err(|e| fail(format!("cannot read {}: {e}", input.display())))?;
        let (mut reviewed, mut current, mut elsewhere) = (Vec::new(), 0, Vec::new());
        for trace in traces {
            let Some(stamp) = suspect::stamp(&graph, trace) else { continue };
            if trace.attributes.get("reviewed").and_then(|v| v.as_string()) == Some(stamp.as_str()) {
                current += 1;
                continue;
            }
            let value = source_edit::string_literal(&stamp);
            match source_edit::set_trace_attribute(&text, &trace.from, &trace.trace_type, &trace.to, "reviewed", &value) {
                Ok(edited) => {
                    text = edited;
                    reviewed.push(format!("{} {} {}", trace.from, trace.trace_type, trace.to));
                }
                Err(source_edit::EditError::NotFound(_)) => elsewhere.push(format!("{} {} {}", trace.from, trace.trace_type, trace.to)),
                Err(e) => return Err(fail(e.to_string())),
            }
        }
        for (case, requirements) in cases {
            let designator = if case.id.is_empty() { &case.name } else { &case.id };
            let entries = suspect::reviewed_entries(&graph, case, &requirements).map_err(fail)?;
            let written: Vec<String> = match case.attributes.get("reviewed") {
                Some(crate::compiler::ast::AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string().map(str::to_string)).collect(),
                _ => Vec::new(),
            };
            if entries == written {
                current += requirements.len();
                continue;
            }
            let value = format!("[{}]", entries.iter().map(|e| source_edit::string_literal(e)).collect::<Vec<_>>().join(", "));
            match source_edit::set_attribute(&text, designator, "reviewed", &value) {
                Ok(edited) => {
                    text = edited;
                    reviewed.extend(requirements.iter().map(|r| format!("{designator} verifies {r}")));
                }
                Err(source_edit::EditError::NotFound(_)) => elsewhere.extend(requirements.iter().map(|r| format!("{designator} verifies {r}"))),
                Err(e) => return Err(fail(e.to_string())),
            }
        }
        if !reviewed.is_empty() {
            source_edit::compile_beside(input, &text).map_err(|e| fail(format!("the reviewed model would not compile: {e}")))?;
            if !print {
                std::fs::write(input, &text).map_err(|e| fail(format!("cannot write {}: {e}", input.display())))?;
            }
        }
        if print {
            print!("{text}");
            return Ok(());
        }
        println!("Reviewed {} link(s) in {}{}", reviewed.len(), input.display(), if current > 0 { format!(" ({current} already up to date)") } else { String::new() });
        for line in &reviewed {
            println!("  ✓ {line}");
        }
        for line in &elsewhere {
            println!("  - {line}: written in another file (or as `trace {{ from: ... }}`); review it there");
        }
        Ok(())
    }

    /// The generated header and source `input` designates: a directory
    /// holding them, or one of the two files.
    fn generated_c(input: &std::path::Path) -> Result<(PathBuf, PathBuf), CliError> {
        let is_header = |path: &std::path::Path| {
            path.extension().is_some_and(|e| e == "h") && std::fs::read_to_string(path).is_ok_and(|text| text.contains("Generated by ArcLang from model") && text.contains("/* state_machine \""))
        };
        let header = if input.is_dir() {
            let mut headers: Vec<PathBuf> = std::fs::read_dir(input).map_err(CliError::Io)?.flatten().map(|entry| entry.path()).filter(|path| is_header(path)).collect();
            headers.sort();
            match headers.len() {
                1 => headers.remove(0),
                0 => return Err(CliError::Compilation(format!("{} holds no header the compiler generated for state machines (`arclang export -f c` writes `<model>.h` and `<model>.c`)", input.display()))),
                _ => return Err(CliError::Compilation(format!("{} holds the C of several models ({}): give the header of one", input.display(), headers.iter().map(|h| h.display().to_string()).collect::<Vec<_>>().join(", ")))),
            }
        } else {
            input.with_extension("h")
        };
        let code = header.with_extension("c");
        for path in [&header, &code] {
            if !path.is_file() {
                return Err(CliError::Compilation(format!("{} is not there: the machines are read from `<model>.h` and `<model>.c` together", path.display())));
            }
        }
        Ok((header, code))
    }

    fn run_prove(&self, model: &std::path::Path, only: Option<&str>, seconds: u64, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        use crate::compiler::recover::correspondence::{self, Outcome};
        let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(model).map_err(|e| CliError::Compilation(e.to_string()))?;
        let base = model.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let work = keep.clone().unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-prove-{}-{}", std::process::id(), crate::compiler::simulation::tool::run_number())));
        let outcomes = correspondence::prove(&compiled.ast, &base, only, &work, seconds).map_err(CliError::Compilation);
        if keep.is_none() {
            let _ = std::fs::remove_dir_all(&work);
        }
        let outcomes = outcomes?;
        if json {
            let rows: Vec<serde_json::Value> = outcomes.iter().map(|(c, outcome)| serde_json::json!({ "correspondence": c.name, "machine": c.machine, "sources": c.sources, "outcome": outcome })).collect();
            println!("{}", serde_json::to_string_pretty(&rows).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            println!("Proof of {}", model.display());
            for (c, outcome) in &outcomes {
                let of = format!("correspondence \"{}\": {} is state_machine \"{}\"", c.name, c.sources.join(", "), c.machine);
                match outcome {
                    Outcome::Proven { seconds, period_us, int_bits, unrelated } => {
                        println!("✓ Proven ({seconds:.1} s) {of}");
                        println!("    for one loop() a step of {} ms, an int of {int_bits} bits and a long of 32, the pins the correspondence names, and the C subset of the Arduino language", period_us / 1000);
                        if !unrelated.is_empty() {
                            println!("    the proof says nothing of {} of the machine: no line of the correspondence speaks of {}", if unrelated.len() == 1 { "this variable" } else { "these variables" }, unrelated.join(", "));
                        }
                    }
                    Outcome::Refuted { obligation, lines, case, seconds } => {
                        println!("✗ Refuted ({seconds:.1} s) {of}");
                        println!("    {}", match obligation.as_str() {
                            "start" => "after setup() and one loop(), and after init and one step, the code and the model do not correspond",
                            _ => "from two states that correspond, one loop() and one step can leave two that do not",
                        });
                        if lines.is_empty() {
                            println!("    no line of the correspondence fails alone: the proof could not say which");
                        }
                        for line in lines {
                            println!("    does not hold after it: {line}");
                        }
                        if let Some(case) = case {
                            println!("    a case that shows it (the code's side and the model's side of each line):");
                            for line in case.said() {
                                println!("      {line}");
                            }
                        }
                    }
                    Outcome::Unknown { reason } => println!("? Neither proven nor refuted: {of}\n    {reason}"),
                }
            }
        }
        let unproven = outcomes.iter().filter(|(_, outcome)| !matches!(outcome, Outcome::Proven { .. })).count();
        if unproven == 0 {
            Ok(())
        } else {
            Err(CliError::Compilation(format!("{unproven} of {} correspondence(s) not proven", outcomes.len())))
        }
    }

    fn run_drift(&self, model: &std::path::Path, code: &std::path::Path, hdl: Option<&std::path::Path>, json: bool, prove: Option<u64>) -> Result<(), CliError> {
        use crate::compiler::recover;
        let design = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(model).map_err(|e| CliError::Compilation(e.to_string()))?;
        let (header_path, code_path) = Self::generated_c(code)?;
        let header = std::fs::read_to_string(&header_path).map_err(CliError::Io)?;
        let source = std::fs::read_to_string(&code_path).map_err(CliError::Io)?;
        let name = |path: &std::path::Path| path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let (header_name, code_name) = (name(&header_path), name(&code_path));
        let built = recover::Generated { header_name: &header_name, header: &header, code_name: &code_name, code: &source };
        let mut drift = recover::drift::drift(&design.ast, &built).map_err(CliError::Compilation)?;
        // The other files of the export, beside the machines': the I/O
        // interface, the glue of each binding, the host implementation.
        let beside = |dir: &std::path::Path| {
            let dir = dir.to_path_buf();
            move |name: &str| std::fs::read_to_string(dir.join(name)).ok()
        };
        let code_dir = header_path.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let (c_files, _) = crate::compiler::embedded::export(&design.ast).map_err(CliError::Compilation)?;
        drift.files = recover::drift::files(&c_files, &[header_name.as_str(), code_name.as_str()], &beside(&code_dir));
        // The programmable logic, when its directory is given: every file the model generates for it.
        let mut logic_proofs: Vec<(String, recover::logic_equivalence::LogicEquivalence)> = Vec::new();
        if let Some(dir) = hdl {
            if !dir.is_dir() {
                return Err(CliError::Compilation(format!("{} is not a directory: --hdl takes the directory `arclang export -f vhdl` wrote", dir.display())));
            }
            let (hdl_files, _) = crate::compiler::embedded::hdl::export(&design.ast).map_err(|e| CliError::Compilation(format!("the model generates no VHDL to compare with: {e}")))?;
            let mut compared = recover::drift::files(&hdl_files, &[], &beside(dir));
            if compared.iter().all(|file| file.missing) {
                return Err(CliError::Compilation(format!("{} holds none of the files the model generates for the logic ({})", dir.display(), hdl_files.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", "))));
            }
            // The proof of the logic: each entity whose VHDL differs (all of
            // them when the package they share does), as built against as generated.
            if let Some(seconds) = prove {
                let is_package = |name: &str| name.ends_with("_pkg.vhd");
                let package_differs = compared.iter().any(|file| file.differs() && is_package(&file.file));
                let entities: Vec<String> = compared.iter().filter(|file| file.file.ends_with(".vhd") && !is_package(&file.file) && !file.missing && (file.differs() || package_differs)).map(|file| file.file.trim_end_matches(".vhd").to_string()).collect();
                let built_files: Vec<(String, String)> = hdl_files.iter().filter(|(name, _)| name.ends_with(".vhd")).filter_map(|(name, _)| Some((name.clone(), std::fs::read_to_string(dir.join(name)).ok()?))).collect();
                for entity in entities {
                    let work = std::env::temp_dir().join(format!("arclang-logic-equivalence-{}-{}", std::process::id(), crate::compiler::simulation::tool::run_number()));
                    std::fs::create_dir_all(&work).map_err(CliError::Io)?;
                    let outcome = recover::logic_equivalence::prove(&hdl_files, &built_files, &entity, &work, seconds);
                    let _ = std::fs::remove_dir_all(&work);
                    logic_proofs.push((entity, outcome.map_err(CliError::Compilation)?));
                }
            }
            for file in &mut compared {
                file.file = dir.join(&file.file).display().to_string();
            }
            drift.files.extend(compared);
        }
        // The proof: whatever the text of the code, does one step of it do what one step of the model's code does?
        let equivalence = match prove {
            None => None,
            Some(seconds) => {
                let (header, code) = (c_files.iter().find(|(name, _)| name == &header_name), c_files.iter().find(|(name, _)| name == &code_name));
                let (Some((_, header)), Some((_, code))) = (header, code) else {
                    return Err(CliError::Compilation(format!("the model generates no {header_name} and {code_name} to prove the code against")));
                };
                let generated = recover::Generated { header_name: &header_name, header, code_name: &code_name, code };
                let work = std::env::temp_dir().join(format!("arclang-equivalence-{}-{}", std::process::id(), crate::compiler::simulation::tool::run_number()));
                let outcome = recover::equivalence::prove(&generated, &built, &work, seconds).map_err(CliError::Compilation)?;
                let _ = std::fs::remove_dir_all(&work);
                Some(outcome)
            }
        };
        if json {
            let mut value = serde_json::to_value(&drift).map_err(|e| CliError::Compilation(e.to_string()))?;
            if let (Some(outcome), Some(object)) = (&equivalence, value.as_object_mut()) {
                object.insert("equivalence".to_string(), serde_json::to_value(outcome).map_err(|e| CliError::Compilation(e.to_string()))?);
            }
            if let (false, Some(object)) = (logic_proofs.is_empty(), value.as_object_mut()) {
                let rows: Vec<serde_json::Value> = logic_proofs.iter().map(|(entity, outcome)| serde_json::json!({ "entity": entity, "outcome": outcome })).collect();
                object.insert("logic_equivalence".to_string(), serde_json::Value::Array(rows));
            }
            println!("{}", serde_json::to_string_pretty(&value).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            println!("Drift of {} and {} against {}", header_path.display(), code_path.display(), model.display());
            let compared = drift.files.iter().filter(|file| !file.missing).count();
            if drift.identical && drift.none() {
                println!("✓ No drift: the machines' code is, byte for byte, the code the model generates{}", if compared > 0 { format!(", and so are the {compared} other file(s) compared") } else { String::new() });
            } else if drift.identical {
                println!("  The machines' code is, byte for byte, the code the model generates.");
            } else if drift.none() {
                println!("✓ No drift: the machines of the code are those of the model, and every line of the code is explained by them");
            }
            let mut machine = None;
            for finding in &drift.findings {
                if machine != Some(&finding.machine) {
                    machine = Some(&finding.machine);
                    if !finding.machine.is_empty() {
                        println!("  state_machine \"{}\"", finding.machine);
                    }
                }
                let place = match (&finding.file, finding.line) {
                    (Some(file), Some(line)) => format!(" ({file}:{line})"),
                    _ => String::new(),
                };
                println!("    ✗ {}{place}", finding.what);
            }
            if drift.unexplained_lines > 0 {
                println!("  {} line(s) of the code are not what its machines generate (edited by hand, or a comment that does not say what the code does):", drift.unexplained_lines);
                for (file, line, text) in &drift.unexplained {
                    println!("    {file}:{line}: {text}");
                }
            }
            for file in drift.files.iter().filter(|file| file.differs()) {
                println!("    ✗ {}:{}: {} line(s) of the file as built where the model generates {}", file.file, file.line, file.built.len(), file.generated.len());
                for line in &file.built {
                    println!("        built: {line}");
                }
                for line in &file.generated {
                    println!("        model: {line}");
                }
            }
            for file in drift.files.iter().filter(|file| file.missing) {
                println!("  note: {} is not among the files as built: not compared", file.file);
            }
            match &equivalence {
                None => {}
                Some(recover::equivalence::Equivalence::Proven { comparisons, seconds }) => {
                    println!("✓ Proven equivalent ({seconds:.1} s): from any state and for any inputs, one step of the code as built gives the state and the outputs one step of the model's code gives ({comparisons} fields compared)");
                }
                Some(recover::equivalence::Equivalence::Different { witness, seconds }) => {
                    println!("✗ Not equivalent ({seconds:.1} s): a state and inputs from which one step of the code as built does not give what the model's code gives:");
                    for (field, value) in witness {
                        println!("    {field} = {value}");
                    }
                }
                Some(recover::equivalence::Equivalence::Unknown { reason }) => println!("  ? Equivalence neither proven nor refuted: {reason}"),
            }
            for (entity, outcome) in &logic_proofs {
                use recover::logic_equivalence::LogicEquivalence;
                match outcome {
                    LogicEquivalence::Proven { points, dropped, seconds } => {
                        println!("✓ Proven equivalent ({seconds:.1} s) entity {entity}: from reset and for any inputs, the logic as built and the logic the model generates agree on {points} signals, their outputs among them, at every clock");
                        if *dropped > 0 {
                            println!("    {dropped} anonymous wire(s) the synthesis numbered alike are not compared: they are not the same wire in the two");
                        }
                    }
                    LogicEquivalence::Different { cycle, outputs, seconds } => {
                        println!("✗ Not equivalent ({seconds:.1} s) entity {entity}: {cycle} clock(s) after reset, with the same inputs, the logic as built does not give what the model's gives:");
                        for (output, model, built) in outputs {
                            println!("    {output} = {built} as built, {model} for the model");
                        }
                    }
                    LogicEquivalence::Unknown { reason } => println!("  ? Equivalence of entity {entity} neither proven nor refuted: {reason}"),
                }
            }
        }
        // A proof of equivalence is the last word on the machines' code: it
        // was edited, and it does the same. Without it, a difference is a drift.
        let proven = matches!(equivalence, Some(recover::equivalence::Equivalence::Proven { .. }));
        let refuted = matches!(equivalence, Some(recover::equivalence::Equivalence::Different { .. }));
        // A VHDL file that differs and whose entity is proven equivalent is not a drift;
        // neither is the package, once every entity that uses it is proven.
        use recover::logic_equivalence::LogicEquivalence;
        let logic_refuted = logic_proofs.iter().any(|(_, outcome)| matches!(outcome, LogicEquivalence::Different { .. }));
        let logic_all_proven = !logic_proofs.is_empty() && logic_proofs.iter().all(|(_, outcome)| matches!(outcome, LogicEquivalence::Proven { .. }));
        let entity_proven = |file: &str| logic_proofs.iter().any(|(entity, outcome)| file.ends_with(&format!("{entity}.vhd")) && matches!(outcome, LogicEquivalence::Proven { .. }));
        let other_files = drift.files.iter().any(|file| file.differs() && !entity_proven(&file.file) && !(file.file.ends_with("_pkg.vhd") && logic_all_proven));
        let machines_same = drift.findings.is_empty() && drift.unexplained_lines == 0;
        let refuted = refuted || logic_refuted;
        if (drift.none() || ((proven || machines_same) && !other_files)) && !refuted {
            Ok(())
        } else {
            let files = drift.files.iter().filter(|file| file.differs()).count();
            Err(CliError::Compilation(format!(
                "the code has drifted from the model: {} difference(s) in its machines, {} line(s) they do not explain, {files} other file(s) that differ",
                drift.findings.len(),
                drift.unexplained_lines
            )))
        }
    }

    fn run_recover(&self, input: &std::path::Path, output: Option<&std::path::Path>, check: bool, hdl: Option<&std::path::Path>) -> Result<(), CliError> {
        use crate::compiler::recover;
        let (header_path, code_path) = Self::generated_c(input)?;
        let header = std::fs::read_to_string(&header_path).map_err(CliError::Io)?;
        let code = std::fs::read_to_string(&code_path).map_err(CliError::Io)?;
        let name = |path: &std::path::Path| path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let (header_name, code_name) = (name(&header_path), name(&code_path));
        let generated = recover::Generated { header_name: &header_name, header: &header, code_name: &code_name, code: &code };
        let mut recovered = recover::from_generated_c(&generated).map_err(CliError::Compilation)?;
        // The machines the logic computes, sized as its register maps say.
        if let Some(dir) = hdl {
            let mut maps = Vec::new();
            let mut names: Vec<PathBuf> = std::fs::read_dir(dir).map_err(CliError::Io)?.flatten().map(|entry| entry.path()).filter(|path| path.file_name().is_some_and(|n| n.to_string_lossy().ends_with("_regs.json"))).collect();
            names.sort();
            for path in &names {
                let text = std::fs::read_to_string(path).map_err(CliError::Io)?;
                maps.push(recover::logic::read_map(&text, &path.display().to_string()).map_err(CliError::Compilation)?);
            }
            if maps.is_empty() {
                return Err(CliError::Compilation(format!("{} holds no register map (`<block>_regs.json`): --hdl takes the directory `arclang export -f vhdl` wrote", dir.display())));
            }
            recover::logic::apply(&mut recovered, &maps).map_err(CliError::Compilation)?;
        }
        let source = recover::to_arclang(&recovered, &generated);
        match output {
            Some(path) => {
                std::fs::write(path, &source).map_err(CliError::Io)?;
                eprintln!("Recovered model \"{}\" from {} and {}", recovered.model, header_path.display(), code_path.display());
                for machine in &recovered.machines {
                    eprintln!("  state_machine \"{}\": {} state(s), {} transition(s), {} variable(s), {} equation(s)", machine.name, machine.states.len(), machine.transitions.len(), machine.variables.len(), machine.equations.len());
                }
                eprintln!("  Output: {}", path.display());
            }
            None => print!("{source}"),
        }
        for note in &recovered.notes {
            eprintln!("  note: {note}");
        }
        if !check {
            eprintln!("  Not proven: `--check` generates the C of this model again and compares it with the C read.");
            return Ok(());
        }
        let outcome = recover::check(&source, &generated);
        for (macro_name, read, again) in &outcome.periods {
            eprintln!("  note: {macro_name} is {read} in the code and {again} for the recovered model: the period comes from the io_binding of the model, which this C does not hold");
        }
        if outcome.proven() {
            eprintln!("✓ Proven: the recovered model generates {header_name} and {code_name} as they were read");
            let Some(dir) = hdl else { return Ok(()) };
            // The logic: every file the recovered model generates for it, against the file read.
            let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
            let compared = recover::logic::check(&source, &read).map_err(|error| CliError::Compilation(format!("the logic is not proven: {error}")))?;
            let unlike: Vec<&recover::drift::FileDrift> = compared.iter().filter(|file| file.missing || file.differs()).collect();
            if unlike.is_empty() {
                eprintln!("✓ Proven: the recovered model generates the {} file(s) of the logic in {} as they were read", compared.len(), dir.display());
                return Ok(());
            }
            eprintln!("✗ Not proven: {} of the {} file(s) of the logic differ from what the recovered model generates", unlike.len(), compared.len());
            for file in unlike {
                if file.missing {
                    eprintln!("  {}: not in {}", file.file, dir.display());
                    continue;
                }
                eprintln!("  {}:{}: {} line(s) read where the recovered model generates {}", file.file, file.line, file.built.len(), file.generated.len());
                for line in file.built.iter().take(3) {
                    eprintln!("    read:        {}", line.trim());
                }
                for line in file.generated.iter().take(3) {
                    eprintln!("    regenerated: {}", line.trim());
                }
            }
            return Err(CliError::Compilation("the recovered model is not the logic".to_string()));
        }
        if let Some(error) = &outcome.error {
            eprintln!("✗ Not proven: {error}");
        } else {
            eprintln!("✗ Not proven: {} line(s) of the C differ from what the recovered model generates", outcome.differing_lines);
            for difference in &outcome.differences {
                eprintln!("  {}:{}\n    read:        {}\n    regenerated: {}", difference.file, difference.line, difference.read, difference.regenerated);
            }
        }
        Err(CliError::Compilation("the recovered model is not the code".to_string()))
    }

    fn run_trace(
        &self,
        input: PathBuf,
        from: Option<String>,
        to: Option<String>,
        validate: bool,
        matrix: bool,
    ) -> Result<(), CliError> {
        println!("Analyzing traceability in {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                if validate {
                    let warnings = result.semantic_model.validate_traceability();
                    if warnings.is_empty() {
                        println!("✓ All elements properly traced");
                    } else {
                        println!("⚠ Traceability issues found:");
                        for warning in &warnings {
                            println!("  {}", warning);
                        }
                    }
                }
                
                // --from / --to keep the traces of those ends, given by id or
                // by name; they list the matrix even without --matrix.
                let filtered = from.is_some() || to.is_some();
                let graph = crate::compiler::elements::build(&result.ast, &result.semantic_model);
                let names = |written: &str| -> Vec<String> {
                    match crate::compiler::suspect::end(&graph, written) {
                        Some(e) => vec![e.id.clone(), e.name.clone()],
                        None => vec![written.to_string()],
                    }
                };
                let wanted = |end: &str, filter: &Option<String>| filter.as_deref().is_none_or(|f| f == end || names(f).iter().any(|n| n == end) || names(end).iter().any(|n| n == f));
                if matrix || filtered {
                    println!("\nTraceability Matrix:");
                    println!("═══════════════════════════════════════");
                    let traces: Vec<_> = result.semantic_model.traces.iter().filter(|t| wanted(&t.from, &from) && wanted(&t.to, &to)).collect();
                    if filtered && traces.is_empty() {
                        return Err(CliError::Compilation("no trace matches: give --from and --to as the ids or names of its ends".to_string()));
                    }
                    for trace in traces {
                        println!("  {} → {}", trace.from, trace.to);
                        if let Some(ref rationale) = trace.rationale {
                            println!("    Rationale: {}", rationale);
                        }
                    }
                }
                
                let metrics = result.semantic_model.compute_metrics();
                println!("\nTraceability Coverage: {:.1}%", metrics.traceability_coverage);
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Trace analysis failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_check_tools(&self, json: bool) -> Result<(), CliError> {
        use crate::compiler::simulation::tool::{diagnose, ToolConfig};
        let diagnosis = diagnose(&ToolConfig::from_env());
        if json {
            println!("{}", serde_json::to_string_pretty(&diagnosis).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            println!("{}", diagnosis.summary());
        }
        Ok(())
    }

    fn run_statistics(&self, input: &std::path::Path, test: Option<&str>, engine: &str, json: bool) -> Result<(), CliError> {
        use crate::compiler::simulation::{replay, statistics};
        let engine = statistics::Engine::parse(engine).ok_or_else(|| CliError::Config(format!("--statistics takes r or python, not '{engine}'")))?;
        let test = test.ok_or_else(|| CliError::Config("--statistics needs --test CASE, the Monte Carlo case whose samples to analyze".into()))?;
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let case = result.ast.test_cases.iter().find(|t| t.id == test || t.name == test).ok_or_else(|| CliError::Config(format!("no test_case '{test}'")))?;
        let report = statistics::analyze(&replay::case_dir(&replay::dir(input), &case.id), engine).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&report).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            println!("Statistics of the samples of {} — {}", case.id, input.display());
            statistics::summary(&report).iter().for_each(|line| println!("{line}"));
            println!("
  The verdict and the gate are the compiler's: `arclang verify` and `arclang gate`.");
        }
        Ok(())
    }

    fn run_verify(&self, input: PathBuf, test: Option<String>, json: bool) -> Result<(), CliError> {
        use crate::compiler::simulation::{run, tool::ToolConfig};
        let result = crate::Compiler::new(crate::CompilerConfig::default())
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;
        let outcomes = run::verify(&result, &input, test.as_deref(), &ToolConfig::from_env()).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&outcomes).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            println!("Verification by simulation — {}", input.display());
            if let Some(first) = outcomes.iter().find(|o| !o.tool.is_empty()) {
                println!("  Tool: {}{}", first.tool, if first.omc_version.is_empty() { String::new() } else { format!(", {}", first.omc_version) });
            }
            for outcome in &outcomes {
                let mark = match outcome.verdict.as_str() {
                    "pass" => "✓",
                    "fail" => "✗",
                    _ => "?",
                };
                let name = if outcome.name.is_empty() || outcome.name == outcome.test_case { String::new() } else { format!(" \"{}\"", outcome.name) };
                println!("\n{mark} {}{name}: {}", outcome.test_case, outcome.verdict.to_uppercase());
                println!("    {}", crate::compiler::simulation::context_line(outcome));
                let print_value = |indent: &str, value: &crate::compiler::simulation::expects::Evaluated| {
                    let margin = if value.margin.is_empty() { String::new() } else { format!(", {}", value.margin) };
                    println!("{indent}{} {}   ({}{margin})", if value.holds { "ok  " } else { "FAIL" }, value.value, value.expect);
                };
                if let Some(required) = outcome.probability {
                    let failing: Vec<_> = outcome.points.iter().filter(|p| !p.holds).collect();
                    println!("    Monte Carlo: {} samples, {} hold; required: {} % at {} % confidence", outcome.points.len(), outcome.points.len() - failing.len(), required.probability * 100.0, required.confidence * 100.0);
                    println!("    {}", outcome.reason);
                    for point in failing.iter().take(10) {
                        println!("    {} {}", if point.error.is_empty() { "FAIL" } else { "ERR " }, point.label);
                        point.values.iter().filter(|v| !v.holds).for_each(|v| print_value("        ", v));
                        if !point.error.is_empty() {
                            println!("        {}", point.error);
                        }
                    }
                    if failing.len() > 10 {
                        println!("    ... and {} more (--json lists every sample)", failing.len() - 10);
                    }
                    let mut expects: Vec<&str> = Vec::new();
                    for influence in &outcome.sensitivity {
                        if !expects.contains(&influence.expect.as_str()) {
                            expects.push(&influence.expect);
                        }
                    }
                    for expect in expects {
                        let strong: Vec<_> = outcome.sensitivity.iter().filter(|i| i.expect == expect && i.rank_correlation.abs() >= 0.1).collect();
                        if strong.is_empty() {
                            continue;
                        }
                        println!("    what moves \"{expect}\" (rank correlation with its margin):");
                        strong.iter().for_each(|i| println!("        {:+.2}  {}", i.rank_correlation, i.parameter));
                    }
                } else if outcome.points.is_empty() {
                    outcome.values.iter().for_each(|v| print_value("    ", v));
                } else {
                    let passing = outcome.points.iter().filter(|p| p.holds).count();
                    println!("    sweep: {} points, {passing} pass, {} do not", outcome.points.len(), outcome.points.len() - passing);
                    for point in &outcome.points {
                        println!("    {} {}", if point.holds { "ok  " } else if point.error.is_empty() { "FAIL" } else { "ERR " }, point.label);
                        point.values.iter().for_each(|v| print_value("        ", v));
                        if !point.error.is_empty() {
                            println!("        {}", point.error);
                        }
                    }
                }
                if !outcome.reason.is_empty() && outcome.verdict != "fail" && outcome.probability.is_none() {
                    println!("    {}", outcome.reason);
                }
                if self.verbose && !outcome.log.trim().is_empty() {
                    println!("    OpenModelica log:");
                    for line in outcome.log.lines().filter(|l| !l.trim().is_empty()) {
                        println!("      | {line}");
                    }
                }
            }
            let seconds: f64 = outcomes.iter().map(|o| o.seconds).sum();
            if seconds > 0.0 {
                println!("\n  Simulated in {seconds:.1} s{}", if self.verbose { "" } else { " (-v prints the OpenModelica log)" });
            }
            let cached = outcomes.iter().filter(|o| o.verdict != "inconclusive").count();
            if cached > 0 {
                println!("\n  {cached} verdict(s) cached in {}", crate::compiler::simulation::cache::dir(&input).display());
            }
        }
        let failed = outcomes.iter().filter(|o| o.verdict != "pass").count();
        if failed == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {} simulated case(s) do not pass", outcomes.len())))
        }
    }

    fn run_export_zephyr(&self, input: &std::path::Path, output: &std::path::Path, target: Option<&str>, binding: Option<&str>) -> Result<(), CliError> {
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let app = crate::compiler::embedded::zephyr::export(&result.ast, target, binding).map_err(CliError::Compilation)?;
        for (name, text, executable) in &app.files {
            let path = output.join(name);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(CliError::Io)?;
            }
            std::fs::write(&path, text).map_err(CliError::Io)?;
            #[cfg(unix)]
            if *executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).map_err(CliError::Io)?;
            }
            #[cfg(not(unix))]
            let _ = executable;
        }
        for note in &app.notes {
            println!("  note: {note}");
        }
        println!("✓ Export successful — a Zephyr application of {} file(s) in {}", app.files.len(), output.display());
        println!("  Build: west build -b <board> {} (or {}/run.sh); see docs/embedded-targets.md.", output.display(), output.display());
        Ok(())
    }

    fn run_sil(&self, input: &std::path::Path, test: Option<String>, compilers: Vec<String>, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        prebuild_plants(input, test.as_deref())?;
        use crate::compiler::embedded::sil;
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let compilers = if compilers.is_empty() { vec![std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string())] } else { compilers };
        let scratch = keep.is_none();
        let work = keep.unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-sil-{}", std::process::id())));
        let mut outcomes = Vec::new();
        for compiler in &compilers {
            let version = sil::compiler_version(compiler).ok_or_else(|| CliError::Config(format!("the C compiler '{compiler}' does not run (give one with --cc, or set CC)")))?;
            let options = sil::Options { compiler: compiler.clone(), work: work.join(compiler.replace(['/', '\\'], "_")), only: test.clone() };
            let run = sil::run_with(&result.ast, &options, &mut plants_of(input));
            let run = run.map_err(CliError::Compilation)?;
            if !json {
                println!("\nSIL {} — {compiler} ({version})", input.display());
                for outcome in &run {
                    self.print_sil(outcome);
                }
            }
            outcomes.extend(run);
        }
        if scratch {
            let _ = std::fs::remove_dir_all(&work);
        } else if !json {
            println!("\n  Generated files, harnesses and binaries kept in {}", work.display());
        }
        // Kept where check, gate and the tables read verdicts.
        let cached = sil::cache(&result.ast, input, &outcomes).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&outcomes).unwrap_or_default());
        } else if !cached.is_empty() {
            println!("\n  {} verdict(s) cached in {}", cached.len(), crate::compiler::simulation::cache::dir(input).display());
        }
        let failed = outcomes.iter().filter(|o| o.verdict != "pass").count();
        if failed == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {} SIL run(s) do not pass", outcomes.len())))
        }
    }

    fn run_cosim(&self, input: &std::path::Path, test: Option<String>, compiler: Option<String>, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        use crate::compiler::embedded::cosim;
        // No OMSimulator here: the same command in ArcLang Studio, which has it with OpenModelica and gcc.
        if let Some(code) = crate::compiler::embedded::docker::relay("cosim", input).map_err(CliError::Config)? {
            std::process::exit(code);
        }
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let compiler = compiler.or_else(|| std::env::var("CC").ok().filter(|c| !c.trim().is_empty())).unwrap_or_else(|| "cc".to_string());
        crate::compiler::embedded::sil::compiler_version(&compiler).ok_or_else(|| CliError::Config(format!("the C compiler '{compiler}' does not run (give one with --cc, or set CC)")))?;
        let scratch = keep.is_none();
        let work = keep.unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-cosim-{}", std::process::id())));
        let options = cosim::Options { compiler, work: work.clone(), only: test, base: crate::compiler::simulation::cache::model_dir(input) };
        let outcomes = cosim::run_with(&result.ast, &options, &mut plants_of(input)).map_err(CliError::Compilation)?;
        // Kept where check, gate and the tables read verdicts: evidence of the SIL stage.
        let cached = cosim::cache(&result.ast, input, &outcomes).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&outcomes).unwrap_or_default());
        } else {
            println!("\nCo-simulation {} — {}", input.display(), cosim::version().unwrap_or_default());
            for outcome in &outcomes {
                self.print_stage(outcome, "OMSimulator");
            }
            if !cached.is_empty() {
                println!("\n  {} verdict(s) cached in {}", cached.len(), crate::compiler::simulation::cache::dir(input).display());
            }
        }
        if scratch {
            let _ = std::fs::remove_dir_all(&work);
        } else if !json {
            println!("\n  Each case's SSP (cosim.ssp), result and logs kept in {}", work.display());
        }
        let failed = outcomes.iter().filter(|o| o.verdict != "pass" && o.verdict != "skipped").count();
        let run = outcomes.iter().filter(|o| o.verdict != "skipped").count();
        if failed == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {run} co-simulation(s) do not pass")))
        }
    }

    fn run_pil(&self, input: &std::path::Path, test: Option<String>, target: Option<String>, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        use crate::compiler::embedded::pil;
        // No Zephyr or Renode here: the same command in the SoC image, its verdict in the model's cache.
        prebuild_plants(input, test.as_deref())?;
        if let Some(code) = crate::compiler::embedded::docker::relay("pil", input).map_err(CliError::Config)? {
            std::process::exit(code);
        }
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let scratch = keep.is_none();
        let work = keep.unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-pil-{}", std::process::id())));
        let options = pil::Options { target, work: work.clone(), only: test };
        let mut plants = plants_of(input);
        let (package, outcomes) = pil::run_with(&result.ast, &options, &mut plants).map_err(CliError::Compilation)?;
        if !json {
            println!("\nPIL {} — {}", input.display(), outcomes.first().map(|o| o.compiler.as_str()).unwrap_or(package.name.as_str()));
            for outcome in &outcomes {
                self.print_sil(outcome);
            }
        }
        if scratch {
            let _ = std::fs::remove_dir_all(&work);
        } else if !json {
            println!("\n  Applications, builds, plans and logs kept in {}", work.display());
        }
        let cached = pil::cache(&result.ast, input, &package, &outcomes).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&outcomes).unwrap_or_default());
        } else if !cached.is_empty() {
            println!("\n  {} PIL verdict(s) cached in {}", cached.len(), crate::compiler::simulation::cache::dir(input).display());
        }
        let failed = outcomes.iter().filter(|o| o.verdict != "pass").count();
        if failed == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {} PIL run(s) do not pass", outcomes.len())))
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_hdl(&self, input: &std::path::Path, test: Option<String>, synth: bool, as_if: Option<String>, machine: Option<String>, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        use crate::compiler::embedded::hdl;
        // No GHDL here: the same command in the FPGA image.
        prebuild_plants(input, test.as_deref())?;
        if let Some(code) = crate::compiler::embedded::docker::relay("hdl", input).map_err(CliError::Config)? {
            std::process::exit(code);
        }
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let scratch = keep.is_none();
        let work = keep.unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-hdl-{}", std::process::id())));
        let compiler = std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string());
        let as_if = match as_if {
            None => None,
            Some(text) => {
                if !synth {
                    return Err(CliError::Config("--as-if goes with --synth".to_string()));
                }
                let (part, mhz) = text.split_once('@').map(|(p, m)| (p.to_string(), Some(m.to_string()))).unwrap_or((text.clone(), None));
                crate::compiler::embedded::alloc::device(&part).map_err(CliError::Config)?;
                let clock_hz = match mhz {
                    Some(m) => m.trim().parse::<f64>().ok().filter(|m| *m > 0.0).ok_or_else(|| CliError::Config(format!("--as-if: '{m}' is not a clock in MHz")))? * 1e6,
                    None => crate::compiler::embedded::alloc::DEFAULT_CLOCK_HZ,
                };
                Some((part, clock_hz))
            }
        };
        let options = hdl::Options { work: work.clone(), only: test, synth, compiler, as_if, machine };
        let mut plants = plants_of(input);
        let (outcomes, synthesis) = hdl::run_with(&result.ast, &options, &mut plants).map_err(CliError::Compilation)?;
        if !json {
            println!("\nHDL {} — {}", input.display(), outcomes.first().map(|o| o.tools.as_str()).unwrap_or(""));
            for o in &outcomes {
                self.print_hdl(o, "VHDL");
            }
            for s in &synthesis {
                match s {
                    Ok(s) => {
                        println!("\n  {} synthesis {} on {}", if s.timing_met { "✓" } else { "✗ TIMING NOT MET:" }, s.block, s.device);
                        println!("    {} LUT4, {} flip-flops, {} DSP (MULT18X18D), {} block RAM (DP16KD), {} I/O", s.luts, s.ffs, s.dsps, s.brams, s.ios);
                        println!("    Fmax {:.2} MHz for a clock of {} MHz: {}", s.fmax_mhz, s.clock_mhz, if s.timing_met { "met" } else { "NOT met" });
                        println!("    bitstream {}", s.bitstream);
                        for n in &s.notes {
                            println!("    note: {n}");
                        }
                    }
                    Err(e) => println!("\n  ! ERROR synthesis: {e}"),
                }
            }
        }
        if scratch {
            let _ = std::fs::remove_dir_all(&work);
        } else if !json {
            println!("\n  VHDL, testbenches, C reference and netlists kept in {}", work.display());
        }
        let cached = hdl::cache(&result.ast, input, &outcomes).map_err(CliError::Compilation)?;
        if json {
            let synthesis: Vec<serde_json::Value> = synthesis.iter().map(|s| match s {
                Ok(s) => serde_json::to_value(s).unwrap_or_default(),
                Err(e) => serde_json::json!({ "error": e }),
            }).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "cases": outcomes, "synthesis": synthesis })).unwrap_or_default());
        } else if !cached.is_empty() {
            println!("\n  {} HDL verdict(s) cached in {}", cached.len(), crate::compiler::simulation::cache::dir(input).display());
        }
        let failed = outcomes.iter().filter(|o| o.verdict != "pass").count();
        let unsynthesised = synthesis.iter().filter(|s| !matches!(s, Ok(s) if s.timing_met)).count();
        if failed == 0 && unsynthesised == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {} HDL run(s) do not pass, {unsynthesised} of {} synthesis fail or miss timing", outcomes.len(), synthesis.len())))
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_soc(&self, input: &std::path::Path, test: Option<String>, bitstream: bool, board: Option<String>, simulate: bool, gate_level: bool, port: Option<String>, keep: Option<PathBuf>, json: bool) -> Result<(), CliError> {
        use crate::compiler::embedded::system;
        // No LiteX or Verilator here: the same command in the system image. A board on a port stays on this host.
        if port.is_none() {
            prebuild_plants(input, test.as_deref())?;
        if let Some(code) = crate::compiler::embedded::docker::relay("soc", input).map_err(CliError::Config)? {
                std::process::exit(code);
            }
        }
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let scratch = keep.is_none();
        let work = keep.unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-soc-{}", std::process::id())));
        let compiler = std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string());
        let on_board = port.is_some();
        let options = system::Options { work: work.clone(), only: test, compiler, simulate, gate_level, bitstream, board, port };
        let mut plants = plants_of(input);
        let report = system::run_with(&result.ast, &options, &mut plants).map_err(CliError::Compilation)?;
        let device = if on_board { "board" } else { "SoC" };
        if !json {
            println!("\nSoC {} — {}", input.display(), report.outcomes.first().map(|o| o.tools.as_str()).unwrap_or(""));
            for o in &report.outcomes {
                self.print_hdl(o, device);
            }
            match &report.bitstream {
                Some(Ok(b)) => {
                    println!("\n  {} bitstream for {} ({}): VexRiscv, {} KiB of RAM holding the Zephyr application ({} bytes), {}", if b.timing_met { "✓" } else { "✗ TIMING NOT MET:" }, b.board, b.device, system::RAM_BYTES / 1024, b.image_bytes, b.blocks.join(", "));
                    println!("    {} LUT4, {} flip-flops, {} DSP (MULT18X18D), {} block RAM (DP16KD)", b.luts, b.ffs, b.dsps, b.brams);
                    for (name, fmax, target) in &b.clocks {
                        println!("    clock {name}: Fmax {fmax:.2} MHz for {target:.2} MHz: {}", if fmax >= target { "met" } else { "NOT met" });
                    }
                    println!("    bitstream {}", b.bitstream);
                    println!("    to the board: {}", b.flash);
                }
                Some(Err(e)) => println!("\n  ! ERROR bitstream: {e}"),
                None => {}
            }
        }
        if scratch && !on_board && !bitstream {
            let _ = std::fs::remove_dir_all(&work);
        } else if !json {
            println!("\n  application, SoC, simulator and bitstream kept in {}", work.display());
        }
        let cached = system::cache(&result.ast, input, &report.outcomes).map_err(CliError::Compilation)?;
        if json {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        } else if !cached.is_empty() {
            println!("\n  {} SoC verdict(s) cached in {}", cached.len(), crate::compiler::simulation::cache::dir(input).display());
        }
        let failed = report.outcomes.iter().filter(|o| o.verdict != "pass").count();
        let unbuilt = usize::from(report.bitstream.as_ref().is_some_and(|b| !matches!(b, Ok(b) if b.timing_met)));
        if failed == 0 && unbuilt == 0 {
            Ok(())
        } else {
            Err(CliError::CiFailed(format!("{failed} of {} SoC run(s) do not pass{}", report.outcomes.len(), if unbuilt > 0 { "; the bitstream failed or misses timing" } else { "" })))
        }
    }

    fn print_hdl(&self, o: &crate::compiler::embedded::hdl::Outcome, device: &str) {
        let mark = match o.verdict.as_str() {
            "pass" => "✓ pass ",
            "fail" => "✗ FAIL ",
            _ => "! ERROR",
        };
        let title = if o.name == o.test_case { o.name.clone() } else { format!("{} ({})", o.name, o.test_case) };
        println!("\n  {mark} {title}");
        let mut context = vec![format!("machine {}", o.machine), format!("block {}", o.block)];
        if !o.verifies.is_empty() {
            context.insert(0, format!("verifies {}", o.verifies.join(", ")));
        }
        println!("    {}", context.join(" · "));
        println!("    model: {} · {device}: {}", o.model_verdict, o.hdl_verdict);
        if let Some(c) = &o.cycles {
            println!("    timing: {c}");
        }
        if let Some(b) = &o.bit_exact {
            match &b.first_mismatch {
                None => println!("    {device} vs fixed-point C: {} steps × {} outputs, largest difference {} (bit-exact)", b.steps, b.columns, b.max_difference),
                Some(m) => println!("    {device} vs fixed-point C: MISMATCH {m}"),
            }
        }
        if let Some(q) = &o.quantisation {
            match &q.first_mismatch {
                None if q.column.is_empty() => println!("    fixed point vs model: no Real variable, every output equal"),
                None => println!("    fixed point vs model: largest difference {:.3e} on {} (tolerance {:.3e}, {})", q.max_difference, q.column, q.tolerance, q.origin),
                Some(m) => println!("    fixed point vs model: MISMATCH {m}"),
            }
            for (column, format, d, t) in &q.variables {
                println!("      {column} {format}: {d:.3e} ≤ {t:.3e}");
            }
        }
        for judged in &o.expectations {
            let value = |v: &Option<crate::compiler::simulation::expects::Evaluated>| match v {
                Some(v) => format!("{}{}", v.value.rsplit_once(" = ").map(|(_, x)| x).unwrap_or(&v.value), if v.holds { "" } else { " (fails)" }),
                None => "—".to_string(),
            };
            let holds = judged.c.as_ref().is_some_and(|v| v.holds);
            println!("    {} {}   model {} · {device} {}", if holds { "ok  " } else { "FAIL" }, judged.expect, value(&judged.model), value(&judged.c));
        }
        if !o.reason.is_empty() {
            println!("    {}", o.reason);
        }
    }

    fn print_sil(&self, outcome: &crate::compiler::embedded::sil::Outcome) {
        self.print_stage(outcome, "C");
    }

    /// A run of an implementation (`label`: C, OMSimulator) judged against the model.
    fn print_stage(&self, outcome: &crate::compiler::embedded::sil::Outcome, label: &str) {
        let mark = match outcome.verdict.as_str() {
            "pass" => "✓ pass ",
            "fail" => "✗ FAIL ",
            "skipped" => "– skip ",
            _ => "! ERROR",
        };
        let title = if outcome.name == outcome.test_case { outcome.name.clone() } else { format!("{} ({})", outcome.name, outcome.test_case) };
        println!("\n  {mark} {title}");
        if outcome.verdict == "skipped" {
            println!("    {}", outcome.reason);
            return;
        }
        let mut context = vec![format!("machine {}", outcome.machine)];
        if !outcome.verifies.is_empty() {
            context.insert(0, format!("verifies {}", outcome.verifies.join(", ")));
        }
        if let Some(binding) = &outcome.binding {
            context.push(format!("through io_binding {binding}"));
        }
        println!("    {}", context.join(" · "));
        println!("    model: {} · {label}: {} · {}", outcome.model_verdict, outcome.c_verdict, {
            // `sil` says "the C"; another stage names its run.
            let what = if label == "C" { "the C".to_string() } else { format!("the {label} run") };
            if outcome.agrees { format!("{what} reproduces the model") } else { format!("{what} does NOT reproduce the model") }
        });
        if let Some(timing) = &outcome.timing {
            println!("    timing: {timing}");
        }
        if let Some(b2b) = &outcome.back_to_back {
            match &b2b.first_mismatch {
                None => println!("    back to back: {} steps × {} outputs, largest difference {} (tolerance {})", b2b.steps, b2b.columns, b2b.max_difference, b2b.tolerance),
                Some(m) => println!("    back to back: MISMATCH {m}"),
            }
        }
        for judged in &outcome.expectations {
            let value = |v: &Option<crate::compiler::simulation::expects::Evaluated>| match v {
                Some(v) => format!("{}{}", v.value.rsplit_once(" = ").map(|(_, x)| x).unwrap_or(&v.value), if v.holds { "" } else { " (fails)" }),
                None => "—".to_string(),
            };
            let holds = judged.c.as_ref().is_some_and(|v| v.holds);
            println!("    {} {}   model {} · {label} {}", if holds { "ok  " } else { "FAIL" }, judged.expect, value(&judged.model), value(&judged.c));
        }
        if !outcome.reason.is_empty() {
            println!("    {}", outcome.reason);
        }
    }

    fn run_gate(&self, input: PathBuf, standard: SafetyStandard) -> Result<(), CliError> {
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        let standard_name = match standard {
            SafetyStandard::ISO26262 => "ISO26262",
            SafetyStandard::DO178C => "DO178C",
            SafetyStandard::IEC61508 => "IEC61508",
        };
        let report = crate::compiler::production_gate::run_gate(
            &result.ast,
            &result.semantic_model,
            standard_name,
        );

        println!("Production-readiness gate ({}) — {}", standard_name, input.display());
        println!(
            "  Requirements: {} total, {} satisfied, {} verified",
            report.requirements_total, report.requirements_satisfied, report.requirements_verified
        );
        if report.constraints_total > 0 {
            println!(
                "  Constraints: {} declared, {} hold",
                report.constraints_total, report.constraints_satisfied
            );
        }

        let blockers: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == crate::compiler::production_gate::Severity::Blocker)
            .collect();
        let gate_warnings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == crate::compiler::production_gate::Severity::Warning)
            .collect();

        if !blockers.is_empty() {
            println!("\n✗ {} blocker(s):", blockers.len());
            for finding in &blockers {
                println!("  [{}] {}", finding.check, finding.message);
            }
        }
        if !gate_warnings.is_empty() {
            println!("\n⚠ {} warning(s):", gate_warnings.len());
            for finding in &gate_warnings {
                println!("  [{}] {}", finding.check, finding.message);
            }
        }

        if report.passed {
            println!("\nPRODUCTION GATE: PASS ✓");
            Ok(())
        } else {
            println!("\nPRODUCTION GATE: FAIL ✗ ({} blockers)", blockers.len());
            Err(CliError::Compilation(format!(
                "production gate failed with {} blocker(s)",
                blockers.len()
            )))
        }
    }

    fn run_diff(&self, old: PathBuf, new: PathBuf, json: bool) -> Result<(), CliError> {
        let compile = |path: &PathBuf| -> Result<crate::CompilationResult, CliError> {
            crate::Compiler::new(crate::CompilerConfig::default())
                .compile_file(path)
                .map_err(|e| CliError::Compilation(format!("{}: {e}", path.display())))
        };
        let old_model = compile(&old)?;
        let new_model = compile(&new)?;
        let report = crate::compiler::semantic_diff::diff_compiled(&old_model, &new_model);

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .map_err(|e| CliError::Compilation(e.to_string()))?
            );
        } else {
            println!("Semantic diff (by stable identity): {} -> {}", old.display(), new.display());
            if report.is_empty() {
                println!("  No semantic changes.");
                return Ok(());
            }
            Self::print_diff(&report);
        }

        // Like diff(1): exit 1 when there are differences, so CI can gate on it.
        if report.is_empty() {
            Ok(())
        } else {
            Err(CliError::DiffFound)
        }
    }

    /// Drawn unless `--no-placement`; the workbench layout next to the model,
    /// when there is one, moves the boxes (a layout that cannot be read is
    /// reported and left out: the drawing is automatic then).
    fn run_twin_live(&self, input: PathBuf, only: Option<String>, json: bool, for_seconds: Option<u64>) -> Result<(), CliError> {
        use crate::compiler::simulation::{twin, twin_live};
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| CliError::Compilation(e.to_string()))?;
        let live: Vec<_> = result
            .ast
            .twins
            .iter()
            .filter(|t| only.as_deref().is_none_or(|o| t.id == o || t.name == o))
            .filter(|t| twin::spec_of(&t.attributes).is_ok_and(|s| s.data.live.is_some()))
            .collect();
        let chosen = match live.as_slice() {
            [one] => *one,
            [] => return Err(CliError::Compilation(format!("{} has no twin on live data (data: {{ mqtt: \"mqtt://broker:1883\" topics: [...] }}, or opcua:, kafka:, http:)", input.display()))),
            _ => return Err(CliError::Compilation(format!("{} twins read live data: name one with --twin ({})", live.len(), live.iter().map(|t| t.id.as_str()).collect::<Vec<_>>().join(", ")))),
        };
        let config = crate::compiler::simulation::tool::ToolConfig::from_env();
        let until = for_seconds.map(std::time::Duration::from_secs);
        // Stopped by a line `stop` on the standard input (VS Code), Ctrl-C or SIGTERM: what was
        // received since the last record is written before the end.
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let on_line = stop.clone();
        std::thread::spawn(move || {
            for line in std::io::BufRead::lines(std::io::stdin().lock()).map_while(Result::ok) {
                if line.trim() == "stop" {
                    on_line.store(true, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
            }
        });
        let on_signal = stop.clone();
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() else { return };
            runtime.block_on(async {
                #[cfg(unix)]
                {
                    let Ok(mut term) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) else { return };
                    tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} }
                }
                #[cfg(not(unix))]
                let _ = tokio::signal::ctrl_c().await;
            });
            on_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        twin_live::live(
            &result,
            &input,
            chosen,
            &config,
            until,
            &stop,
            |outcome| {
                if json {
                    println!("{}", serde_json::to_string(outcome).unwrap_or_default());
                } else {
                    let worst = outcome.comparisons.iter().map(|c| format!("{} within {:.1} %", c.target, c.within * 100.0)).collect::<Vec<_>>().join(", ");
                    println!("{} twin '{}': {} over {} samples{}{}", chrono::Local::now().format("%H:%M:%S"), outcome.name, outcome.verdict.to_uppercase(), outcome.samples, if worst.is_empty() { String::new() } else { format!("; {worst}") }, if outcome.reason.is_empty() || outcome.verdict == "pass" { String::new() } else { format!("; {}", outcome.reason) });
                }
                let _ = std::io::Write::flush(&mut std::io::stdout());
            },
            |status| {
                if json {
                    println!("{}", serde_json::to_string(status).unwrap_or_default());
                } else {
                    eprintln!("{}: {}", status.status, status.detail);
                }
                let _ = std::io::Write::flush(&mut std::io::stdout());
            },
        )
        .map_err(CliError::Compilation)
    }

    fn run_twin(&self, input: PathBuf, only: Option<String>, json: bool, list: bool) -> Result<(), CliError> {
        use crate::compiler::simulation::twin;
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&input).map_err(|e| CliError::Compilation(e.to_string()))?;
        if list {
            let twins: Vec<serde_json::Value> = result
                .ast
                .twins
                .iter()
                .map(|t| serde_json::json!({ "id": t.id, "name": t.name, "verifies": t.verifies, "live": twin::spec_of(&t.attributes).is_ok_and(|s| s.data.live.is_some()) }))
                .collect();
            println!("{}", serde_json::Value::Array(twins));
            return Ok(());
        }
        let chosen: Vec<_> = result.ast.twins.iter().filter(|t| only.as_deref().is_none_or(|o| t.id == o || t.name == o)).collect();
        if chosen.is_empty() {
            return Err(CliError::Compilation(match only {
                Some(o) => format!("no twin '{o}' in {}", input.display()),
                None => format!("{} declares no twin (twin \"Name\" {{ simulate: ... data: ... drives: ... }})", input.display()),
            }));
        }
        let config = crate::compiler::simulation::tool::ToolConfig::from_env();
        let outcomes: Vec<twin::Outcome> = chosen.iter().map(|t| twin::run(&result, &input, t, &config)).collect();
        for (twin, outcome) in chosen.iter().zip(&outcomes) {
            if outcome.verdict != "inconclusive" {
                twin::cache(&result, &input, twin, outcome).map_err(CliError::Compilation)?;
            }
        }
        if json {
            println!("{}", serde_json::to_string(&outcomes).map_err(|e| CliError::Compilation(e.to_string()))?);
        } else {
            for o in &outcomes {
                println!("twin '{}' ({}): {}", o.name, o.twin, o.verdict.to_uppercase());
                if !o.data.is_empty() {
                    println!("  data: {}, {} samples over {:.0} s from {}", o.data, o.samples, o.duration, o.data_start);
                }
                for c in &o.comparisons {
                    let unit = crate::compiler::quantity::lookup_unit(&c.unit);
                    println!(
                        "  {} {} ~ {}: max deviation {}, RMS {}, {:.1} % of the samples within {} ({:.1} % required){}",
                        if c.holds { "ok  " } else { "FAIL" },
                        c.target,
                        c.column,
                        twin::shown(c.max_deviation, unit),
                        twin::shown(c.rms_deviation, unit),
                        c.within * 100.0,
                        c.tolerance,
                        c.required * 100.0,
                        c.first_exceedance.map(|t| format!(", first beyond at {t:.0} s")).unwrap_or_default()
                    );
                }
                for v in &o.values {
                    println!("  {} {}: {}", if v.holds { "ok  " } else { "FAIL" }, v.expect, v.value);
                }
                if !o.reason.is_empty() && o.verdict != "pass" {
                    println!("  {}", o.reason);
                }
            }
        }
        if outcomes.iter().any(|o| o.verdict != "pass") {
            return Err(CliError::Compilation(format!("{} of {} twin(s) do not hold", outcomes.iter().filter(|o| o.verdict != "pass").count(), outcomes.len())));
        }
        Ok(())
    }

    fn modelica_options(&self, input: &std::path::Path, no_placement: bool) -> crate::compiler::modelica::Options {
        use crate::compiler::diagram::layout_file::{sidecar_path, LayoutFile};
        if no_placement {
            return crate::compiler::modelica::Options::default();
        }
        let path = sidecar_path(input);
        let layout = match std::fs::read_to_string(&path) {
            Ok(text) => match LayoutFile::parse(&text) {
                Ok(layout) => {
                    println!("  layout: {}", path.display());
                    Some(layout)
                }
                Err(error) => {
                    eprintln!("warning: {}: {error}: the automatic layout is used", path.display());
                    None
                }
            },
            Err(_) => None,
        };
        crate::compiler::modelica::Options { placement: true, layout }
    }

    /// `export -f simulink-connector`: the Capella model (read, or written from the ArcLang
    /// one), the chosen elements made a pivot, the pivot a script; `output.mlcore` and
    /// `output.m` are written.
    fn run_simulink_connector(
        &self,
        input: &std::path::Path,
        output: &std::path::Path,
        strategy: &str,
        elements: &[String],
        options: &crate::compiler::simulink_connector::m2m::Options,
    ) -> Result<(), CliError> {
        use crate::compiler::simulink_connector::{export, m2m};
        let strategy = m2m::Strategy::parse(strategy).ok_or_else(|| CliError::Config(format!("unknown strategy '{strategy}': expected functions, functional-chains, state-machines or data-pkg")))?;
        let model = if input.extension().is_some_and(|e| e == "capella") {
            m2m::model::Model::load(input).map_err(CliError::Compilation)?
        } else {
            let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
            let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "model".to_string());
            let exported = crate::compiler::capella_export::export(&result.ast, &stem);
            let capella = format!("{stem}.capella");
            let aird = format!("{stem}.aird");
            m2m::model::Model::from_strings(&[(capella.as_str(), exported.capella.as_str()), (aird.as_str(), exported.aird.as_str())]).map_err(CliError::Compilation)?
        };
        let exported = export::export(&model, strategy, elements, options).map_err(CliError::Compilation)?;
        let base = match output.extension().and_then(|e| e.to_str()) {
            Some("m") | Some("mlcore") => output.with_extension(""),
            _ => output.to_path_buf(),
        };
        if let Some(parent) = base.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(CliError::Io)?;
        }
        let pivot = base.with_extension("mlcore");
        let script = base.with_extension("m");
        std::fs::write(&pivot, &exported.pivot).map_err(CliError::Io)?;
        std::fs::write(&script, &exported.script).map_err(CliError::Io)?;
        for note in &exported.notes {
            eprintln!("note: {note}");
        }
        println!("✓ Export successful");
        println!("  Pivot: {}", pivot.display());
        println!("  MATLAB script: {}", script.display());
        Ok(())
    }

    fn run_export(
        &self,
        input: PathBuf,
        output: PathBuf,
        format: ExportFormat,
        no_placement: bool,
        split: bool,
    ) -> Result<(), CliError> {
        println!("Exporting {} to {:?} format...", input.display(), format);
        
        let mut config = crate::CompilerConfig::default();
        
        // Set target based on format (Mermaid uses default, others specify target)
        config.target = match format {
            ExportFormat::JSON => "json".to_string(),
            ExportFormat::Capella => "capella".to_string(),
            ExportFormat::XML => "xml".to_string(),
            ExportFormat::Markdown => "markdown".to_string(),
            ExportFormat::Mermaid => "json".to_string(),
            ExportFormat::PlantUML => "json".to_string(),
            ExportFormat::HTML => "json".to_string(),
            ExportFormat::PDF => "json".to_string(),
            ExportFormat::YAML => "json".to_string(),
            ExportFormat::Terraform => "terraform".to_string(),
            ExportFormat::SysML => "json".to_string(),
            ExportFormat::SysMLJson => "json".to_string(),
            ExportFormat::Simulink => "json".to_string(),
            ExportFormat::SimulinkConnector => "json".to_string(),
            ExportFormat::FMI => "json".to_string(),
            ExportFormat::ReqIF => "json".to_string(),
            ExportFormat::CHeaders => "json".to_string(),
            ExportFormat::Proto => "json".to_string(),
            ExportFormat::GeoJSON => "json".to_string(),
            ExportFormat::Step => "json".to_string(),
            ExportFormat::Modelica => "json".to_string(),
            ExportFormat::C => "json".to_string(),
            ExportFormat::Fmu => "json".to_string(),
            ExportFormat::Zephyr => "json".to_string(),
            ExportFormat::Vhdl => "json".to_string(),
        };
        
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                let output_content = match format {
                    ExportFormat::JSON => {
                        // Export the raw AST model as JSON for diagram rendering
                        result.ast.to_json()
                            .map_err(|e| CliError::Compilation(format!("JSON export failed: {}", e)))?
                    }
                    ExportFormat::YAML => {
                        // The model `-f json` writes, as YAML.
                        let json = result.ast.to_json().map_err(|e| CliError::Compilation(format!("YAML export failed: {}", e)))?;
                        let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| CliError::Compilation(format!("YAML export failed: {}", e)))?;
                        crate::compiler::yaml::to_yaml(&value)
                    }
                    ExportFormat::Mermaid => {
                        use crate::compiler::mermaid_generator::generate_mermaid_flowchart;
                        generate_mermaid_flowchart(&result.semantic_model, "System Requirements", "elk")
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::PlantUML => {
                        use crate::compiler::plantuml_generator::generate_plantuml_component;
                        generate_plantuml_component(&result.semantic_model)
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::HTML => {
                        // The explorer is the renderer that actually draws the
                        // diagram (layers, components, ports, labeled edges).
                        // The ELK v2 pipeline's to_html only dumps layout JSON.
                        use crate::compiler::arcviz_explorer::generate_explorer_html_arranged;

                        let mut semantic_model = result.semantic_model.clone();
                        if semantic_model.name.is_none() {
                            // No `model` header: fall back to the file name.
                            semantic_model.name = input
                                .file_stem()
                                .map(|stem| stem.to_string_lossy().to_string());
                        }
                        let (html, _json) = generate_explorer_html_arranged(&semantic_model, &result.ast, Some(&input))
                            .map_err(|e| CliError::Compilation(format!("HTML generation failed: {}", e)))?;
                        html
                    }
                    ExportFormat::Capella => {
                        // A Capella 7 project: the .capella, and the .aird and
                        // .afm beside it that Capella needs to open it.
                        let capella_path = if output.extension().is_some_and(|e| e == "capella") { output.clone() } else { output.with_extension("capella") };
                        let stem = capella_path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "model".to_string());
                        let exported = crate::compiler::capella_export::export(&result.ast, &stem);
                        if let Some(dir) = capella_path.parent().filter(|d| !d.as_os_str().is_empty()) {
                            std::fs::create_dir_all(dir).map_err(CliError::Io)?;
                        }
                        std::fs::write(&capella_path, &exported.capella).map_err(CliError::Io)?;
                        std::fs::write(capella_path.with_extension("aird"), &exported.aird).map_err(CliError::Io)?;
                        std::fs::write(capella_path.with_extension("afm"), &exported.afm).map_err(CliError::Io)?;
                        for note in &exported.notes {
                            println!("  note: {note}");
                        }
                        println!("✓ Export successful");
                        println!("  Output: {} (+ .aird, .afm)", capella_path.display());
                        return Ok(());
                    }
                    ExportFormat::SysML => {
                        // SysML v2 textual notation (interoperability subset)
                        crate::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast)
                    }
                    ExportFormat::SysMLJson => {
                        let text = crate::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
                        let rendered = crate::compiler::sysml_records::from_text(&text)
                            .map_err(|e| CliError::Compilation(format!("SysML v2 abstract syntax: {}", e)))?;
                        serde_json::to_string_pretty(&rendered.records)
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    ExportFormat::CHeaders => {
                        crate::compiler::c_header_generator::generate_c_headers(
                            &result.semantic_model, &result.ast)
                    }
                    ExportFormat::GeoJSON => {
                        let (text, notes) = crate::compiler::spatial::geojson::export(&result.ast);
                        for note in notes {
                            eprintln!("note: {note}");
                        }
                        text
                    }
                    ExportFormat::Step => {
                        let stamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
                        let exported = crate::compiler::spatial::step::export(&result.ast, &stamp)
                            .map_err(CliError::Compilation)?;
                        for note in &exported.notes {
                            eprintln!("note: {note}");
                        }
                        eprintln!("STEP AP242: {} solid(s)", exported.solids);
                        exported.text
                    }
                    ExportFormat::Proto => {
                        crate::compiler::proto_generator::generate_proto(
                            &result.semantic_model, &result.ast)
                    }
                    ExportFormat::ReqIF => {
                        // Requirements exchange with DOORS/Polarion/Jama
                        let (text, notes) = crate::compiler::reqif::generate_reqif_with_notes(&result.semantic_model, &result.ast);
                        for note in notes {
                            eprintln!("warning: {note}");
                        }
                        text
                    }
                    ExportFormat::Simulink => {
                        // MATLAB script rebuilding the architecture in System Composer
                        crate::compiler::simulink_generator::generate_simulink_script(
                            &result.semantic_model,
                            &result.ast,
                        )
                    }
                    ExportFormat::Modelica => {
                        // One package file: `output` is a DIRECTORY here.
                        let options = self.modelica_options(&input, no_placement);
                        let export = crate::compiler::modelica::export_with(&result.ast, &options).map_err(CliError::Compilation)?;
                        std::fs::create_dir_all(&output).map_err(CliError::Io)?;
                        let path = if split {
                            for (file, text) in &export.files {
                                let target = output.join(file);
                                if let Some(parent) = target.parent() {
                                    std::fs::create_dir_all(parent).map_err(CliError::Io)?;
                                }
                                std::fs::write(&target, text).map_err(CliError::Io)?;
                            }
                            // A single-file export of the same package beside the folder
                            // would define it twice: it was written by ArcLang, so it goes.
                            let single = output.join(&export.file_name);
                            if std::fs::read_to_string(&single).is_ok_and(|t| t.starts_with(crate::compiler::modelica::GENERATED_HEADER)) {
                                std::fs::remove_file(&single).map_err(CliError::Io)?;
                                println!("  note: removed {}, the single-file export this folder replaces", single.display());
                            }
                            output.join(&export.package)
                        } else {
                            if output.join(&export.package).is_dir() {
                                println!("  note: {} also holds a split export of {} (--split); OpenModelica would see the package twice", output.display(), export.package);
                            }
                            let path = output.join(&export.file_name);
                            std::fs::write(&path, &export.text).map_err(CliError::Io)?;
                            path
                        };
                        for note in &export.notes {
                            println!("  note: {note}");
                        }
                        println!("✓ Export successful — package {} ({} system model(s))", export.package, export.systems.len());
                        for system in export.systems.values() {
                            println!("  system model: {system}");
                        }
                        println!("  Output: {}", path.display());
                        println!("  The physics goes in a package of its own extending these models (see docs/modelica.md).");
                        return Ok(());
                    }
                    ExportFormat::Vhdl => {
                        let (files, notes) = crate::compiler::embedded::hdl::export(&result.ast).map_err(CliError::Compilation)?;
                        for (name, text) in &files {
                            let path = output.join(name);
                            if let Some(dir) = path.parent() {
                                std::fs::create_dir_all(dir).map_err(CliError::Io)?;
                            }
                            std::fs::write(&path, text).map_err(CliError::Io)?;
                        }
                        for note in &notes {
                            println!("  note: {note}");
                        }
                        println!("✓ Export successful — {} file(s): VHDL-2008, register map, driver, fixed-point C reference", files.len());
                        for (name, _) in &files {
                            println!("  {}", output.join(name).display());
                        }
                        println!("  Simulate and synthesise with `arclang hdl`; see docs/embedded-fpga.md.");
                        return Ok(());
                    }
                    ExportFormat::C => {
                        let (files, notes) = crate::compiler::embedded::export(&result.ast).map_err(CliError::Compilation)?;
                        for (name, text) in &files {
                            let path = output.join(name);
                            if let Some(dir) = path.parent() {
                                std::fs::create_dir_all(dir).map_err(CliError::Io)?;
                            }
                            std::fs::write(&path, text).map_err(CliError::Io)?;
                        }
                        for note in &notes {
                            println!("  note: {note}");
                        }
                        println!("✓ Export successful — {} file(s) of portable C99", files.len());
                        for (name, _) in &files {
                            println!("  {}", output.join(name).display());
                        }
                        println!("  Compile with -std=c99; see docs/embedded-c.md for the API and the I/O layer.");
                        return Ok(());
                    }
                    ExportFormat::Fmu => {
                        let compiler = std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string());
                        let compiler = crate::compiler::embedded::sil::compiler_version(&compiler).map(|_| compiler);
                        std::fs::create_dir_all(&output).map_err(CliError::Io)?;
                        let work = std::env::temp_dir().join(format!("arclang-fmu-{}", std::process::id()));
                        let built = crate::compiler::embedded::export_fmu(&result.ast, compiler.as_deref(), &work);
                        let _ = std::fs::remove_dir_all(&work);
                        let (name, archive, cases, notes) = built.map_err(CliError::Compilation)?;
                        std::fs::write(output.join(&name), &archive).map_err(CliError::Io)?;
                        for (file, text) in &cases {
                            let path = output.join(file);
                            if let Some(dir) = path.parent() {
                                std::fs::create_dir_all(dir).map_err(CliError::Io)?;
                            }
                            std::fs::write(&path, text).map_err(CliError::Io)?;
                        }
                        for note in &notes {
                            println!("  note: {note}");
                        }
                        println!("✓ Export successful — FMI 2.0 co-simulation FMU {}", output.join(&name).display());
                        println!("  {} case file(s) in {} (inputs, and the model-level reference at the end of each step)", cases.len(), output.join("cases").display());
                        return Ok(());
                    }
                    ExportFormat::FMI => {
                        // One modelDescription.xml per component: `output` is a
                        // DIRECTORY here, not a single file.
                        use crate::compiler::fmi_generator::generate_fmi_descriptors;
                        let descriptors =
                            generate_fmi_descriptors(&result.semantic_model, &result.ast);
                        if descriptors.is_empty() {
                            return Err(CliError::Compilation(
                                "FMI export: no component with ports or exchanges found".to_string(),
                            ));
                        }
                        std::fs::create_dir_all(&output).map_err(CliError::Io)?;
                        for descriptor in &descriptors {
                            let dir = output.join(&descriptor.component_id);
                            std::fs::create_dir_all(&dir).map_err(CliError::Io)?;
                            std::fs::write(dir.join("modelDescription.xml"), &descriptor.xml)
                                .map_err(CliError::Io)?;
                            println!(
                                "  ✓ {} ({}) -> {}/modelDescription.xml",
                                descriptor.component_name,
                                descriptor.component_id,
                                dir.display()
                            );
                        }
                        println!("✓ Export successful — {} FMU interface(s)", descriptors.len());
                        println!("  Input: {}", input.display());
                        println!("  Output directory: {}", output.display());
                        return Ok(());
                    }
                    ExportFormat::PDF => {
                        return Err(CliError::NotImplemented(
                            "PDF export not implemented in v3 — use --format html and print to PDF".to_string()
                        ));
                    }
                    ExportFormat::Terraform => {
                        use crate::compiler::terraform_databricks_generator::{generate_terraform_databricks, TerraformConfig};
                        let config = TerraformConfig::default();
                        generate_terraform_databricks(&result.semantic_model, &config)
                            .map_err(|e| CliError::Compilation(e.to_string()))?
                    }
                    _ => result.output
                };
                
                // `-o dir/file`: the directory is made when missing.
                if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent).map_err(CliError::Io)?;
                }
                std::fs::write(&output, &output_content)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Export successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: {:?}", format);
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Export failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn run_import(
        &self,
        input: PathBuf,
        format: ImportFormat,
        output: PathBuf,
    ) -> Result<(), CliError> {
        println!("Importing from {:?}: {}...", format, input.display());
        
        match format {
            ImportFormat::Capella => {
                // The Capella 6/7 XMI reader: every level, the requirements,
                // the data, the state machines, the realizations, and the
                // diagrams of the .aird beside the file. The result compiles
                // or the import fails.
                let imported = crate::compiler::capella_xmi::import_file(&input).map_err(|failure| {
                    // The text that was refused is kept beside the output, for the reader.
                    if let Some(rejected) = &failure.rejected {
                        let kept = output.with_extension("rejected.arc");
                        if std::fs::write(&kept, rejected).is_ok() {
                            eprintln!("  the refused text is in {}", kept.display());
                        }
                    }
                    CliError::Compilation(failure.message)
                })?;
                std::fs::write(&output, &imported.source)
                    .map_err(CliError::Io)?;
                for (what, count) in &imported.counts {
                    println!("  {count} {what}");
                }
                for note in &imported.notes {
                    println!("  note: {note}");
                }
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                
                Ok(())
            }
            ImportFormat::SysML => {
                let imported = crate::compiler::sysml_import::import_file(&input).map_err(|failure| {
                    if let Some(rejected) = &failure.rejected {
                        let kept = output.with_extension("rejected.arc");
                        if std::fs::write(&kept, rejected).is_ok() {
                            eprintln!("  the refused text is in {}", kept.display());
                        }
                    }
                    CliError::Compilation(failure.message)
                })?;
                std::fs::write(&output, &imported.source).map_err(CliError::Io)?;
                for (what, count) in &imported.counts {
                    println!("  {count} {what}");
                }
                for note in &imported.notes {
                    println!("  note: {note}");
                }
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                Ok(())
            }
            ImportFormat::Mermaid => {
                use crate::compiler::mermaid_importer::import_mermaid;
                
                let content = std::fs::read_to_string(&input)
                    .map_err(|e| CliError::Io(e))?;
                
                let arc_code = import_mermaid(&content)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: Mermaid -> ArcLang");
                
                Ok(())
            }
            ImportFormat::ReqIF => {
                let content = std::fs::read_to_string(&input)
                    .map_err(|e| CliError::Io(e))?;

                let arc_code = crate::compiler::reqif::import_reqif(&content)
                    .map_err(CliError::Compilation)?;

                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;

                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: ReqIF -> ArcLang");

                Ok(())
            }
            ImportFormat::PlantUML => {
                use crate::compiler::plantuml_importer::import_plantuml;
                
                let content = std::fs::read_to_string(&input)
                    .map_err(|e| CliError::Io(e))?;
                
                let arc_code = import_plantuml(&content)
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                std::fs::write(&output, arc_code)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Import successful");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output.display());
                println!("  Format: PlantUML -> ArcLang");
                
                Ok(())
            }
            _ => {
                println!("⚠ Format {:?} not yet implemented", format);
                Err(CliError::Config(format!("Import format {:?} not supported yet", format)))
            }
        }
    }
    
    fn run_safety(
        &self,
        input: PathBuf,
        standard: SafetyStandard,
        fmea: bool,
        fta: bool,
        report: bool,
        output: Option<PathBuf>,
    ) -> Result<(), CliError> {
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;
        if fta || report {
            let trees = crate::compiler::fta::trees(&result.ast).map_err(|errors| CliError::Compilation(errors.join("\n")))?;
            if trees.is_empty() {
                return Err(CliError::Compilation("no fault tree: the model declares no hazard (safety_analysis { hazard ... })".to_string()));
            }
            if report {
                let text = crate::compiler::fta::markdown_report(&result.ast, &trees, &format!("{standard:?}"));
                match &output {
                    Some(path) => {
                        std::fs::write(path, text).map_err(CliError::Io)?;
                        println!("✓ Safety report written to {}", path.display());
                    }
                    None => print!("{text}"),
                }
            } else {
                print!("{}", crate::compiler::fta::text_report(&trees));
            }
            // A tree beyond its budget fails the command, as a gate would.
            if trees.iter().any(|t| t.within_budget() == Some(false)) {
                return Err(CliError::Compilation("a fault tree is beyond its budget".to_string()));
            }
            return Ok(());
        }

        println!("Safety analysis ({:?}) of {}...", standard, input.display());

        let safety_blocks = &result.ast.safety_analysis;
        let hazard_count: usize = safety_blocks.iter().map(|s| s.hazards.len()).sum();
        let fmea_count: usize = safety_blocks.iter().map(|s| s.fmea.len()).sum();

        println!("  Safety analysis blocks: {}", safety_blocks.len());
        println!("  Hazards: {}", hazard_count);
        println!("  FMEA entries: {}", fmea_count);

        if hazard_count == 0 && fmea_count == 0 {
            println!("⚠ No safety analysis content found in the model.");
            println!("  Add a 'safety_analysis' block with 'hazard' and 'fmea' entries.");
        }

        if fmea {
            if fmea_count == 0 {
                return Err(CliError::Compilation(
                    "cannot report FMEA: the model contains no fmea entries".to_string(),
                ));
            }
            println!("\nFMEA entries:");
            for block in safety_blocks {
                for entry in &block.fmea {
                    println!("  - {}", entry.name);
                    for (key, value) in &entry.attributes {
                        println!("      {}: {:?}", key, value);
                    }
                }
            }
        }

        Ok(())
    }
    
    fn run_serve(
        &self,
        port: u16,
        models: Vec<PathBuf>,
        history_depth: Option<usize>,
        token_file: Option<PathBuf>,
        users_file: Option<PathBuf>,
        allow_write: bool,
    ) -> Result<(), CliError> {
        use colored::Colorize;

        let mut workspace = crate::web_server::systems_modeling::Workspace::default();
        // The token never comes from the command line: it would show in the
        // process list and in the shell history.
        let token = match &token_file {
            Some(path) => Some(std::fs::read_to_string(path).map_err(|e| {
                CliError::Config(format!("cannot read the token file {}: {e}", path.display()))
            })?),
            None => std::env::var("ARCLANG_API_TOKEN").ok().filter(|token| !token.trim().is_empty()),
        };
        if let Some(token) = &token {
            workspace.require_token(token).map_err(CliError::Config)?;
            println!("   Authentication: bearer token required on every API request");
        }
        if let Some(path) = &users_file {
            let text = std::fs::read_to_string(path).map_err(|e| {
                CliError::Config(format!("cannot read the users file {}: {e}", path.display()))
            })?;
            let users = workspace.add_users(&text).map_err(CliError::Config)?;
            println!("   Authentication: {users} user(s) declared, a bearer token is required on every API request");
        }
        if allow_write {
            workspace.allow_writes().map_err(|reason| {
                CliError::Config(format!(
                    "{reason}: pass --users-file <file>, --token-file <file> or set ARCLANG_API_TOKEN (for instance: openssl rand -hex 32)"
                ))
            })?;
            println!("   Writes: enabled, one git commit per request");
        }
        for model in &models {
            let project = match history_depth {
                Some(depth) => workspace.add_file_with_history(model, depth),
                None => workspace.add_file(model),
            }
            .map_err(CliError::Compilation)?;
            println!(
                "   Project '{}' ({} commit{}) — {}{}/projects/{}",
                project.name,
                project.commits.len(),
                if project.commits.len() == 1 { "" } else { "s" },
                format!("http://127.0.0.1:{}", port),
                crate::web_server::SYSTEMS_MODELING_BASE,
                project.id
            );
        }
        
        println!("{}", "🚀 Starting ArcLang Rust Backend Server".bright_cyan().bold());
        println!("{}", format!("   Port: {}", port).bright_white());
        println!("{}", format!("   Professional 7D Arcadia Diagrams").bright_green());
        println!();
        
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Compilation(format!("Failed to create runtime: {}", e)))?;
        
        runtime.block_on(async {
            crate::web_server::serve(port, workspace).await
        }).map_err(|e| CliError::Compilation(format!("Server error: {}", e)))?;
        
        Ok(())
    }
    
    fn run_lsp(&self, _stdio: bool, port: Option<u16>) -> Result<(), CliError> {
        if port.is_some() {
            return Err(CliError::NotImplemented(
                "TCP mode is not implemented; use stdio (arclang lsp --stdio)".to_string(),
            ));
        }
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Compilation(format!("Failed to create runtime: {}", e)))?;
        runtime.block_on(language_server::run_stdio());
        Ok(())
    }

    fn run_repl(&self, _project: Option<PathBuf>) -> Result<(), CliError> {
        Err(CliError::NotImplemented("the REPL is not implemented yet".to_string()))
    }

    fn run_clean(&self, _project: PathBuf, _cache: bool) -> Result<(), CliError> {
        Err(CliError::NotImplemented("'clean' is not implemented yet".to_string()))
    }
    
    fn run_info(
        &self,
        input: PathBuf,
        metrics: bool,
        dependencies: bool,
    ) -> Result<(), CliError> {
        if dependencies {
            return Err(CliError::NotImplemented(
                "dependency analysis is not implemented yet".to_string(),
            ));
        }

        println!("Model information: {}", input.display());

        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        let result = compiler
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        if let Some(name) = result.ast.attributes.get("name") {
            println!("  Name: {:?}", name);
        }
        if let Some(version) = result.ast.attributes.get("version") {
            println!("  Version: {:?}", version);
        }

        let model_metrics = result.semantic_model.compute_metrics();
        println!("  Total elements: {}", model_metrics.total_elements);
        println!("  Requirements: {}", model_metrics.requirements_count);
        println!("  Components: {}", model_metrics.components_count);
        println!("  Functions: {}", result.semantic_model.functions.len());
        println!("  Traces: {}", model_metrics.traces_count);

        if metrics {
            println!("  Traceability coverage: {:.1}%", model_metrics.traceability_coverage);
        }

        Ok(())
    }
    
    fn run_explorer(
        &self,
        input: PathBuf,
        output: Option<PathBuf>,
        open: bool,
    ) -> Result<(), CliError> {
        println!("🔍 Generating Architecture Explorer from {}...", input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                use crate::compiler::arcviz_explorer::generate_explorer_html_arranged;

                let mut semantic_model = result.semantic_model.clone();
                if semantic_model.name.is_none() {
                    // No `model` header: fall back to the file name.
                    semantic_model.name = input
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string());
                }
                let (html, json) = generate_explorer_html_arranged(&semantic_model, &result.ast, Some(&input))
                    .map_err(|e| CliError::Compilation(e.to_string()))?;
                
                let output_html = output.unwrap_or_else(|| {
                    input.with_file_name(format!("{}_explorer.html", 
                        input.file_stem().unwrap().to_str().unwrap()))
                });
                
                let output_json = output_html.with_extension("json");
                
                std::fs::write(&output_html, &html)
                    .map_err(|e| CliError::Io(e))?;
                std::fs::write(&output_json, &json)
                    .map_err(|e| CliError::Io(e))?;
                
                println!("✓ Architecture Explorer generated successfully");
                println!("  Input: {}", input.display());
                println!("  Output: {}", output_html.display());
                println!("  Data: {}", output_json.display());
                println!("  Requirements: {}", result.semantic_model.requirements.len());
                println!("  Components: {}", result.semantic_model.components.len());
                println!("  Interfaces: {}", result.semantic_model.interfaces.len());
                println!("  Functions: {}", result.semantic_model.functions.len());
                
                println!("\n📋 Features:");
                println!("  • Interactive architecture diagram");
                println!("  • Expandable requirements & components");
                println!("  • Complete traceability matrix");
                println!("  • Floating table of contents");
                println!("  • PDF & HTML export");
                
                if open {
                    println!("\n🌐 Opening explorer in browser...");
                    opener::open(&output_html)
                        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Explorer generation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    /// Write the viewpoint diagram model (or its ELK layout graphs) as JSON
    /// and print the model inconsistencies found while building it.
    fn run_viewpoint_diagrams(
        &self,
        input: PathBuf,
        output: PathBuf,
        view: Option<String>,
        format: DiagramFormat,
        layout: Option<PathBuf>,
        theme: Option<String>,
    ) -> Result<(), CliError> {
        use crate::compiler::diagram::{build_diagrams, elk, html, layout_file, theme as themes, ViewKind};

        // A theme is checked before anything is compiled: a refused one stops here.
        let theme = theme.as_deref().map(themes::resolve).transpose().map_err(CliError::Config)?;

        let cesam = view.as_deref().map(|v| v.eq_ignore_ascii_case("cesam")).unwrap_or(false);
        let selected = match view.as_deref() {
            None => None,
            Some(_) if cesam => None,
            Some(text) => Some(ViewKind::parse(text).ok_or_else(|| {
                CliError::Compilation(format!("unknown view '{text}' — expected oab, sab, lab, pab, msm, es, cap, cdb, pbs, req, par or cesam"))
            })?),
        };
        let result = crate::Compiler::new(crate::CompilerConfig::default())
            .compile_file(&input)
            .map_err(|e| CliError::Compilation(e.to_string()))?;

        let mut set = build_diagrams(&result.ast);
        crate::compiler::diagram::judge_requirements(&mut set, &result.ast, &result.semantic_model);
        // The layout is checked against every view before one is selected:
        // a view left out by `--view` is not a stale entry of the file.
        layout_file::arrange(&mut set, &input, layout.as_deref()).map_err(CliError::Compilation)?;
        if cesam {
            let graph = crate::compiler::elements::build(&result.ast, &result.semantic_model);
            crate::compiler::cesam::arrange_diagrams(&mut set, &result.ast, &graph);
        }
        if let Some(kind) = selected {
            set.diagrams.retain(|diagram| diagram.kind == kind);
            let (whole, named) = (format!("[{}]", kind.id()), format!("[{}:", kind.id()));
            set.diagnostics.retain(|diagnostic| {
                diagnostic.starts_with(&whole) || diagnostic.starts_with(&named) || diagnostic.starts_with("[layout]")
            });
            if set.diagrams.is_empty() {
                return Err(CliError::Compilation(format!(
                    "{} declares nothing to draw in the {} view",
                    input.display(),
                    kind.id()
                )));
            }
        }

        let serialized = match format {
            DiagramFormat::Elk => {
                let graphs: Vec<serde_json::Value> = set.diagrams.iter().map(elk::to_elk).collect();
                serde_json::to_string_pretty(&graphs).map(|json| format!("{json}\n"))
            }
            DiagramFormat::Html => {
                let title = match result.ast.attributes.get("name") {
                    Some(crate::compiler::ast::AttributeValue::String(name)) => name.clone(),
                    _ => input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
                };
                Ok(html::standalone_html_themed(&set, &title, theme.as_ref()))
            }
            _ => serde_json::to_string_pretty(&set).map(|json| format!("{json}\n")),
        }
        .map_err(|e| CliError::Compilation(e.to_string()))?;
        std::fs::write(&output, serialized).map_err(CliError::Io)?;

        for diagram in &set.diagrams {
            println!("✓ {} — {} nodes, {} edges", diagram.id, count_nodes(&diagram.nodes), diagram.edges.len());
        }
        for diagnostic in &set.diagnostics {
            println!("⚠ {diagnostic}");
        }
        if set.layout.is_some() {
            println!("  Layout: {}", layout.unwrap_or_else(|| layout_file::sidecar_path(&input)).display());
        }
        if let Some(theme) = &theme {
            println!("  Theme: {}", theme["name"].as_str().unwrap_or("custom"));
        }
        println!("  Output: {}", output.display());
        Ok(())
    }

    fn run_diagram(
        &self,
        input: PathBuf,
        output: PathBuf,
        format: DiagramFormat,
        title: String,
        open: bool,
    ) -> Result<(), CliError> {
        println!("🎨 Generating {:?} diagram from {}...", format, input.display());
        
        let config = crate::CompilerConfig::default();
        let mut compiler = crate::Compiler::new(config);
        
        match compiler.compile_file(&input) {
            Ok(result) => {
                match format {
                    DiagramFormat::Mermaid => {
                        use crate::compiler::mermaid_generator::generate_mermaid_flowchart;
                        let diagram = generate_mermaid_flowchart(&result.semantic_model, &title, "elk")
                            .map_err(|e| CliError::Compilation(e.to_string()))?;
                        
                        std::fs::write(&output, &diagram)
                            .map_err(|e| CliError::Io(e))?;
                        
                        println!("✓ Mermaid diagram generated");
                        println!("  Output: {}", output.display());
                        
                        if open {
                            self.open_mermaid_diagram(&output)?;
                        }
                    }
                    
                    DiagramFormat::PlantUML => {
                        use crate::compiler::plantuml_generator::generate_plantuml_component;
                        let diagram = generate_plantuml_component(&result.semantic_model)
                            .map_err(|e| CliError::Compilation(e.to_string()))?;

                        std::fs::write(&output, &diagram).map_err(CliError::Io)?;

                        println!("✓ PlantUML diagram generated");
                        println!("  Output: {}", output.display());
                    }

                    DiagramFormat::All => {
                        self.generate_all_capella_diagrams(&input, &result, &output)?;
                    }
                    
                    _ => {
                        self.generate_capella_diagram(&input, &result, &output, format)?;
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Diagram generation failed: {}", e);
                Err(CliError::Compilation(e.to_string()))
            }
        }
    }
    
    fn generate_capella_diagram(
        &self,
        input: &PathBuf,
        result: &crate::CompilationResult,
        output: &PathBuf,
        format: DiagramFormat,
    ) -> Result<(), CliError> {
        // Step 1: Export AST to JSON
        let json_data = result.ast.to_json()
            .map_err(|e| CliError::Compilation(format!("JSON export failed: {}", e)))?;
        
        let temp_json = std::env::temp_dir().join(format!("arclang_model_{}.json", std::process::id()));
        std::fs::write(&temp_json, &json_data)
            .map_err(|e| CliError::Io(e))?;
        
        // Step 2: Determine diagram service path
        let Some(diagram_service_dir) = diagram_service::locate() else {
            let _ = std::fs::remove_file(&temp_json);
            return Err(CliError::Config(diagram_service::missing()));
        };
        
        // Step 3: Call Node.js diagram renderer
        let diagram_type = match format {
            DiagramFormat::Operational => "operational",
            DiagramFormat::Functional => "functional",
            DiagramFormat::Sequence => "sequence",
            DiagramFormat::StateMachine => "statemachine",
            DiagramFormat::Component => "component",
            DiagramFormat::Physical => "physical",
            DiagramFormat::Class => "class",
            DiagramFormat::Tree => "tree",
            DiagramFormat::Capability => "capability",
            DiagramFormat::FunctionalChain => "functional-chain",
            _ => return Err(CliError::Config(format!("Unsupported format: {:?}", format)))
        };
        
        let test_script = format!("test-{}.js", diagram_type);
        let script_path = diagram_service_dir.join(&test_script);
        
        if !script_path.exists() {
            return Err(CliError::Config(
                format!("Diagram renderer script not found: {}", test_script)
            ));
        }
        
        println!("  📊 Rendering {} diagram...", diagram_type);
        
        // Convert output to absolute path
        let abs_output = std::fs::canonicalize(output.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(|e| CliError::Io(e))?
            .join(output.file_name().unwrap_or_else(|| std::ffi::OsStr::new("output.svg")));
        
        let node_output = std::process::Command::new("node")
            .current_dir(&diagram_service_dir)
            .arg(&test_script)
            .arg(&temp_json)
            .arg(&abs_output)
            .output()
            .map_err(|e| CliError::Config(format!("node could not run the SVG diagram renderer ({e}): install Node.js, or use -f html, viewpoints or elk")))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_json);
        
        if !node_output.status.success() {
            let stderr = String::from_utf8_lossy(&node_output.stderr);
            return Err(CliError::Compilation(format!("Diagram rendering failed: {}", stderr)));
        }
        
        println!("✓ {} diagram generated successfully", diagram_type.to_uppercase());
        println!("  Input: {}", input.display());
        println!("  Output: {}", output.display());
        
        Ok(())
    }
    
    fn generate_all_capella_diagrams(
        &self,
        input: &PathBuf,
        result: &crate::CompilationResult,
        base_output: &PathBuf,
    ) -> Result<(), CliError> {
        println!("📦 Generating all Capella diagrams...\n");
        
        let base_name = base_output.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("diagram");
        
        let output_dir = base_output.parent().unwrap_or_else(|| Path::new("."));
        
        let diagram_types = vec![
            (DiagramFormat::Operational, "operational"),
            (DiagramFormat::Functional, "functional"),
            (DiagramFormat::Sequence, "sequence"),
            (DiagramFormat::StateMachine, "statemachine"),
            (DiagramFormat::Component, "component"),
            (DiagramFormat::Physical, "physical"),
            (DiagramFormat::Class, "class"),
            (DiagramFormat::Tree, "tree"),
            (DiagramFormat::Capability, "capability"),
            (DiagramFormat::FunctionalChain, "functional-chain"),
        ];
        
        let mut success_count = 0;
        let total = diagram_types.len();
        
        for (format, name) in diagram_types {
            let output_path = output_dir.join(format!("{}_{}.svg", base_name, name));
            
            match self.generate_capella_diagram(input, result, &output_path, format) {
                Ok(_) => {
                    success_count += 1;
                    println!();
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to generate {} diagram: {}\n", name, e);
                }
            }
        }
        
        println!("═══════════════════════════════════════");
        println!("✓ Generated {}/{} diagrams successfully", success_count, total);
        println!("  Output directory: {}", output_dir.display());
        
        Ok(())
    }
    
    fn open_mermaid_diagram(&self, mermaid_file: &PathBuf) -> Result<(), CliError> {
        let content = std::fs::read_to_string(mermaid_file)
            .map_err(|e| CliError::Io(e))?;
        
        // Create HTML wrapper
        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ArcLang Diagram</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <script>
        mermaid.initialize({{ startOnLoad: true, theme: 'default' }});
    </script>
    <style>
        body {{
            margin: 0;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #f5f5f5;
        }}
        .container {{
            max-width: 100%;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-top: 0;
        }}
        .mermaid {{
            display: flex;
            justify-content: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 ArcLang Diagram</h1>
        <div class="mermaid">
{}
        </div>
    </div>
</body>
</html>"#, content);
        
        let html_file = mermaid_file.with_extension("html");
        std::fs::write(&html_file, html)
            .map_err(|e| CliError::Io(e))?;
        
        // Open in browser
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg(&html_file)
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open")
            .arg(&html_file)
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        #[cfg(target_os = "windows")]
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", html_file.to_str().unwrap()])
            .spawn()
            .map_err(|e| CliError::Io(e))?;
        
        Ok(())
    }
}

/// The .arc files under `path`: the file itself, or a directory's tree.
/// Hidden directories and `target` are skipped.
fn collect_arc_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), CliError> {
    if !path.is_dir() {
        files.push(path.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?.path();
        let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if entry.is_dir() {
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                collect_arc_files(&entry, files)?;
            }
        } else if entry.extension().is_some_and(|ext| ext == "arc") {
            files.push(entry);
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Compilation error: {0}")]
    Compilation(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// `format --check` found files that are not in formatted form.
    #[error("{0} file(s) not formatted; run `arclang fmt --write`")]
    Unformatted(usize),

    /// Not a failure: the two models differ (diff(1) convention, exit 1).
    #[error("semantic differences found")]
    DiffFound,

    /// A CI/CD command found the model, or the pull request, not fit to go.
    #[error("{0}")] // cicd hook
    CiFailed(String),

    /// A merge left conflicts, or a result that does not compile.
    #[error("{0}")] // semgit hook
    MergeConflict(String),
}

fn count_nodes(nodes: &[crate::compiler::diagram::Node]) -> usize {
    nodes.iter().map(|node| 1 + count_nodes(&node.children)).sum()
}

/// The plants of the model's cases, beside the model.
fn plants_of(input: &std::path::Path) -> crate::compiler::embedded::plant::Plants {
    crate::compiler::embedded::plant::Plants::at(&crate::compiler::simulation::cache::model_dir(input))
}

/// Build the FMU of every plant of the cases chosen, here, where
/// OpenModelica is: a stage that then runs in a Docker image steps it.
fn prebuild_plants(input: &std::path::Path, only: Option<&str>) -> Result<(), CliError> {
    use crate::compiler::embedded::{binding, plan};
    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
    let has_plant = result.ast.test_cases.iter().filter(|t| only.is_none_or(|o| t.id == o || t.name == o)).any(|t| matches!(t.attributes.get("execute"), Some(crate::compiler::ast::AttributeValue::Map(m)) if m.contains_key("plant")));
    if !has_plant {
        return Ok(());
    }
    let p = plan::plan(&result.ast).map_err(CliError::Compilation)?;
    let bindings = binding::resolve_all(&result.ast, &p).map_err(CliError::Compilation)?;
    let built = plants_of(input).prepare_all(&result.ast, &p, &bindings, only).map_err(CliError::Compilation)?;
    for path in built {
        eprintln!("plant: {}", path.display());
    }
    Ok(())
}
