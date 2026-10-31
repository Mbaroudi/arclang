//! Replaying a simulation: the results of a run are kept trimmed beside the
//! model, read back with each part's values, and each expectation says the
//! first instant it fails. No OpenModelica here: the result is written as
//! OpenModelica writes it.

use arclang::compiler::simulation::expects;
use arclang::compiler::simulation::replay;
use arclang::compiler::simulation::results::Series;
use std::collections::BTreeMap;

/// Ten minutes of a sagging bus: the voltage drops 0.5 V a minute from 27 V.
fn run() -> (String, String) {
    let mut csv = String::from("\"time\",\"essentialLoads.voltage\",\"essentialLoads.load.internal\",\"mainBattery.soc\",\"der(mainBattery.soc)\"\n");
    for minute in 0..=10 {
        let v = 27.0 - 0.5 * minute as f64;
        csv.push_str(&format!("{},{v},1,{},0\n", minute * 60, 0.8 - 0.01 * minute as f64));
    }
    let xml = r#"<ScalarVariable name = "essentialLoads.voltage"><Real unit = "V" /></ScalarVariable><ScalarVariable name = "mainBattery.soc"><Real unit = "1" /></ScalarVariable>"#.to_string();
    (csv, xml)
}

fn parts() -> BTreeMap<String, String> {
    BTreeMap::from([("PN-ESS".to_string(), "essentialLoads".to_string()), ("PN-BAT".to_string(), "mainBattery".to_string())])
}

fn aliases() -> BTreeMap<String, String> {
    BTreeMap::from([("PN-ESS".to_string(), "essentialLoads".to_string()), ("PN-BAT".to_string(), "mainBattery".to_string())])
}

#[test]
fn an_extremum_fails_at_the_first_instant_it_breaks_its_bound() {
    let (csv, _) = run();
    let series = Series::from_csv(&csv).unwrap();
    let below = expects::parse("REQ-ESS: min(\"PN-ESS\".voltage) >= 24 V", &aliases()).unwrap();
    // 27 - 0.5 m < 24 from m = 7 (23.5 V) on.
    assert_eq!(expects::first_failure(&below, &series), Some(420.0));
    let holds = expects::parse("min(\"PN-ESS\".voltage) >= 20 V", &aliases()).unwrap();
    assert_eq!(expects::first_failure(&holds, &series), None);
    let at = expects::parse("at(5 min, \"PN-ESS\".voltage) >= 25 V", &aliases()).unwrap();
    assert_eq!(expects::first_failure(&at, &series), Some(300.0));
    let last = expects::parse("final(\"PN-BAT\".soc) >= 0.75", &aliases()).unwrap();
    assert_eq!(expects::first_failure(&last, &series), Some(600.0), "a final value fails at the end of the run");
}

#[test]
fn the_results_are_kept_trimmed_and_read_back_by_part() {
    let dir = tempfile::tempdir().unwrap();
    let (csv, xml) = run();
    let parsed = vec![expects::parse("REQ-ESS: min(\"PN-ESS\".voltage) >= 24 V", &aliases()).unwrap()];
    replay::save(dir.path(), "TC-LOSS", "key-1", &[String::new()], &[(Some(csv), xml)], &parts(), &parsed).unwrap();
    assert!(replay::kept(dir.path(), "TC-LOSS"));
    let kept = std::fs::read_to_string(dir.path().join("TC-LOSS/0.csv")).unwrap();
    let header = kept.lines().next().unwrap();
    assert_eq!(header, "\"time\",\"essentialLoads.voltage\",\"mainBattery.soc\"", "nested and derivative variables are left out");

    let replay = replay::load(dir.path(), "TC-LOSS", 0, "key-1", &parts(), &parsed).unwrap();
    assert!(replay.points.is_empty(), "a single run has no point to choose");
    assert!(!replay.stale);
    assert_eq!(replay.time.len(), 11);
    let ess = replay.elements.iter().find(|e| e.id == "PN-ESS").unwrap();
    assert_eq!((ess.values[0].name.as_str(), ess.values[0].unit.as_str()), ("voltage", "V"));
    assert_eq!(ess.values[0].series[7], 23.5);
    let expectation = &replay.expectations[0];
    assert_eq!((expectation.holds, expectation.first_failure, expectation.element.as_deref(), expectation.requirement.as_deref()), (false, Some(420.0), Some("PN-ESS"), Some("REQ-ESS")));

    // The model changed since the run: the replay says so.
    assert!(replay::load(dir.path(), "TC-LOSS", 0, "key-2", &parts(), &parsed).unwrap().stale);
    // A point the run does not have, a case never run.
    assert!(replay::load(dir.path(), "TC-LOSS", 3, "key-1", &parts(), &parsed).is_err());
    let never = replay::load(dir.path(), "TC-OTHER", 0, "key-1", &parts(), &parsed).err().unwrap();
    assert!(never.contains("run `arclang verify` on the model first"), "{never}");
}

#[test]
fn a_long_run_is_kept_at_most_six_hundred_steps() {
    let mut csv = String::from("\"time\",\"essentialLoads.voltage\"\n");
    for step in 0..5000 {
        csv.push_str(&format!("{step},{}\n", 27.0 - step as f64 * 1e-3));
    }
    let series = Series::from_csv(&csv).unwrap();
    let trimmed = series.trimmed(&|_| true, replay::STEPS);
    assert_eq!(trimmed.len(), replay::STEPS);
    assert_eq!(trimmed.times().last(), Some(4999.0), "the last instant is kept");
}
