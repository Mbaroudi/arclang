//! Tasks and scheduling: several io_bindings on one target, one task each
//! in the exported application, `arclang schedule` on the step times
//! measured (or given). The application itself builds and runs where
//! Zephyr is (the SoC image, CI job "Embedded SoC").

use arclang::compiler::embedded::{binding, plan, sil, zephyr};
use arclang::compiler::partition::{Entry, Software};
use arclang::compiler::schedule;
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

const BODY: &str = "automotive/body-controller/body_controller.arc";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-scheduling-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cc() -> String {
    std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string())
}

#[test]
fn the_priorities_follow_the_periods_unless_written() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(BODY)).unwrap();
    let p = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &p).unwrap();
    assert_eq!(bindings.iter().map(|b| (b.name.as_str(), b.period_us, b.priority)).collect::<Vec<_>>(), vec![("Wiper task", 20_000, None), ("Light task", 100_000, None)]);
    assert_eq!(binding::priorities(&bindings), vec![1, 2], "the shortest period the highest");
    // Written priorities come first; the others follow by period after them.
    let text = std::fs::read_to_string(example(BODY)).unwrap().replace("    period: 100 ms\n", "    period: 100 ms\n    priority: 1\n");
    let ast = Compiler::new(CompilerConfig::default()).compile_string(&text).unwrap().ast;
    let bindings = binding::resolve_all(&ast, &p).unwrap();
    assert_eq!(binding::priorities(&bindings), vec![2, 1]);
    let bad = std::fs::read_to_string(example(BODY)).unwrap().replace("    period: 100 ms\n", "    period: 100 ms\n    priority: 0\n");
    let e = Compiler::new(CompilerConfig::default()).compile_string(&bad).map(|_| ()).unwrap_err().to_string();
    assert!(e.contains("priority is a whole number from 1 (the highest) to 32"), "{e}");
}

#[test]
fn the_c_steps_each_machine_alone_and_the_model_as_their_sequence() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(BODY)).unwrap();
    let p = plan::plan(&result.ast).unwrap();
    let generated = arclang::compiler::embedded::cgen::generate(&p, 20_000);
    assert!(generated.header.contains("void body_controller_wiper_step(body_controller_state *state, const body_controller_inputs *inputs, body_controller_outputs *outputs);"), "{}", generated.header);
    assert!(generated.header.contains("void body_controller_interior_light_step("));
    assert!(generated.header.contains("bool wiper_started;") && generated.header.contains("bool interior_light_started;"));
    assert!(generated.source.contains("void body_controller_step(body_controller_state *state, const body_controller_inputs *inputs, body_controller_outputs *outputs)\n{\n    body_controller_wiper_step(state, inputs, outputs);\n    body_controller_interior_light_step(state, inputs, outputs);\n}"), "{}", generated.source);
    // The glue of a binding steps its machine, not the whole model.
    let bindings = binding::resolve_all(&result.ast, &p).unwrap();
    let glue = arclang::compiler::embedded::glue::generate(&p, &bindings[1]);
    assert!(glue.source.contains("body_controller_interior_light_step(state, inputs, outputs);"), "{}", glue.source);
    assert!(!glue.source.contains("body_controller_step(state"));
    // And the C still reproduces the model, case by case.
    let options = sil::Options { compiler: cc(), work: work("sil"), only: None };
    let outcomes = sil::run(&result.ast, &options).unwrap();
    assert_eq!(outcomes.len(), 2);
    for o in &outcomes {
        assert_eq!(o.verdict, "pass", "{}: {}", o.test_case, o.reason);
    }
}

#[test]
fn the_application_runs_one_task_per_binding() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(BODY)).unwrap();
    let app = zephyr::export(&result.ast, None, None).unwrap();
    let file = |name: &str| app.files.iter().find(|(n, _, _)| n == name).map(|(_, t, _)| t.clone()).unwrap_or_else(|| panic!("{name} in {:?}", app.files.iter().map(|f| &f.0).collect::<Vec<_>>()));
    let tasks = file("src/tasks.c");
    assert!(tasks.contains("K_THREAD_DEFINE(arclang_task_0, ARCLANG_TASK_STACK, task_0, NULL, NULL, NULL, 0, 0, 0);"), "{tasks}");
    assert!(tasks.contains("K_THREAD_DEFINE(arclang_task_1, ARCLANG_TASK_STACK, task_1, NULL, NULL, NULL, 1, 0, 0);"), "{tasks}");
    assert!(tasks.contains("arclang_io_task_wait_period(0U, 20000UL)") && tasks.contains("arclang_io_task_wait_period(1U, 100000UL)"), "{tasks}");
    assert!(tasks.contains("body_controller_wiper_task_cycle(&glue_0") && tasks.contains("body_controller_light_task_cycle(&glue_1"), "{tasks}");
    let header = file("src/arclang_app.h");
    assert!(header.contains("#define ARCLANG_TASKS 2") && header.contains("#include \"body_controller_wiper_task_io.h\"") && header.contains("#include \"body_controller_light_task_io.h\""), "{header}");
    assert!(file("arclang_app.cmake").contains("set(ARCLANG_MAIN tasks CACHE STRING"));
    assert!(app.notes.iter().any(|n| n.contains("task 'Wiper task' runs state_machine 'Wiper' every 20000 us at priority 1")), "{:?}", app.notes);
    assert!(app.notes.iter().any(|n| n.contains("task 'Light task' runs state_machine 'Interior light' every 100000 us at priority 2")), "{:?}", app.notes);
    // One binding named: the controller's loop as before.
    let one = zephyr::export(&result.ast, None, Some("Wiper task")).unwrap();
    assert!(one.files.iter().all(|(n, _, _)| n != "src/tasks.c"));
    assert!(one.files.iter().find(|(n, _, _)| n == "arclang_app.cmake").unwrap().1.contains("set(ARCLANG_MAIN main CACHE STRING"));
}

fn measured(machine: &str, step_us: f64) -> (Entry, bool) {
    (
        Entry {
            machine: machine.to_string(),
            ident: machine.to_lowercase().replace(' ', "_"),
            period_us: 20_000,
            now: "software".into(),
            node: None,
            software: Some(Software { target: "zephyr/litex_vexriscv".into(), step_us, cases: 1, text: None, data: None, bss: None, tools: "gcc".into() }),
            software_error: None,
            logic: None,
            logic_error: None,
            key: "k".into(),
        },
        true,
    )
}

#[test]
fn the_schedule_is_analysed_from_the_measured_steps() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(BODY)).unwrap();
    let p = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &p).unwrap();
    let a = schedule::analyse(&p, &bindings, "zephyr/litex_vexriscv", &[measured("Wiper", 18.4), measured("Interior light", 11.0)], &[]);
    assert_eq!(a.tasks.iter().map(|t| (t.priority, t.wcet_us, t.response_us, t.meets)).collect::<Vec<_>>(), vec![(1, Some(18.4), Some(18.4), Some(true)), (2, Some(11.0), Some(29.4), Some(true))]);
    assert_eq!(a.schedulable, Some(true));
    assert!((a.utilisation - (18.4 / 20_000.0 + 11.0 / 100_000.0)).abs() < 1e-12);
    // A step as long as its period: the task below misses.
    let b = schedule::analyse(&p, &bindings, "zephyr/litex_vexriscv", &[measured("Wiper", 19_000.0), measured("Interior light", 6_000.0)], &[]);
    assert_eq!(b.tasks[1].meets, Some(false), "{:?}", b.tasks[1]);
    assert_eq!(b.schedulable, Some(false));
    assert!(schedule::lines(&b).iter().any(|l| l.contains("MISSES ITS PERIOD")));
    // Unknown: unproved, and the line says what to run; a given time fills it.
    let c = schedule::analyse(&p, &bindings, "zephyr/litex_vexriscv", &[measured("Wiper", 18.4)], &[]);
    assert_eq!(c.schedulable, None);
    assert!(c.tasks[1].wcet_source.contains("run arclang partition"), "{}", c.tasks[1].wcet_source);
    let d = schedule::analyse(&p, &bindings, "zephyr/litex_vexriscv", &[measured("Wiper", 18.4)], &[("Interior light".into(), 12.5)]);
    assert_eq!((d.tasks[1].wcet_us, d.tasks[1].wcet_source.as_str(), d.schedulable), (Some(12.5), "given", Some(true)));
}

#[test]
fn the_command_and_the_table_say_what_is_measured_and_what_is_not() {
    let exe = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(exe).arg("schedule").arg(example(BODY)).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("step time unknown") && text.contains("run arclang partition"), "{text}");
    let out = Command::new(exe).arg("schedule").arg(example(BODY)).args(["--wcet", "Wiper=30", "--wcet", "Interior light=12.5"]).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("every task meets its period") && text.contains("[given]"), "{text}");
    let out = Command::new(exe).arg("schedule").arg(example(BODY)).args(["--wcet", "Wiper=30", "--wcet", "Interior light=12.5", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schedulable"], true);
    assert_eq!(json["tasks"][1]["response_us"], 42.5);
    let out = Command::new(exe).arg("table").arg(example(BODY)).args(["--kind", "schedule", "-f", "csv"]).output().unwrap();
    let csv = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{csv}{}", String::from_utf8_lossy(&out.stderr));
    assert!(csv.contains("Wiper task") && csv.contains("Light task") && csv.contains("run arclang partition"), "{csv}");
}
