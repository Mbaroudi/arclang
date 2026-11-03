//! A Capella project holds the architecture levels it was given: one
//! without an operational or a system analysis comes back without them,
//! while an ArcLang model gets the five levels of a new Capella project.

use arclang::compiler::capella_export::export;
use arclang::compiler::capella_xmi::import_strings;
use arclang::compiler::{Compiler, CompilerConfig};

const ONLY_LOGICAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<org.polarsys.capella.core.data.capellamodeller:Project xmlns:xmi="http://www.omg.org/XMI" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:org.polarsys.capella.core.data.capellamodeller="http://www.polarsys.org/capella/core/modeller/7.0.0" xmlns:org.polarsys.capella.core.data.la="http://www.polarsys.org/capella/core/la/7.0.0" xmlns:org.polarsys.capella.core.data.pa="http://www.polarsys.org/capella/core/pa/7.0.0" id="p1" name="Partial">
  <ownedModelRoots xsi:type="org.polarsys.capella.core.data.capellamodeller:SystemEngineering" id="se1" name="Partial">
    <ownedArchitectures xsi:type="org.polarsys.capella.core.data.la:LogicalArchitecture" id="la1" name="Logical Architecture"/>
    <ownedArchitectures xsi:type="org.polarsys.capella.core.data.pa:PhysicalArchitecture" id="pa1" name="Physical Architecture"/>
  </ownedModelRoots>
</org.polarsys.capella.core.data.capellamodeller:Project>
"#;

fn compile(source: &str) -> arclang::compiler::ast::Model {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}")).ast
}

#[test]
fn a_project_without_some_levels_comes_back_without_them() {
    let first = import_strings(ONLY_LOGICAL, None, "Partial.capella").unwrap_or_else(|f| panic!("{}", f.message));
    let capella = export(&compile(&first.source), "Partial").capella;
    assert!(!capella.contains("oa:OperationalAnalysis") && !capella.contains("ctx:SystemAnalysis"), "{capella}");
    assert!(capella.contains("id=\"la1\"") && capella.contains("id=\"pa1\""), "{capella}");
    assert!(!capella.contains("SystemAnalysisRealization"), "no realization of a missing level: {capella}");
    let second = import_strings(&capella, None, "Partial.capella").unwrap_or_else(|f| panic!("{}", f.message));
    let levels = |s: &str| s.lines().filter(|l| l.ends_with('{') && !l.starts_with(' ')).map(str::to_string).collect::<Vec<_>>();
    assert_eq!(levels(&first.source), levels(&second.source));
}

#[test]
fn an_arclang_model_gets_the_five_levels_of_a_new_project() {
    let capella = export(&compile("model M { name: \"m\" }\nlogical_architecture \"LA\" { component A { id: \"LC-A\" } }\n"), "m").capella;
    for kind in ["oa:OperationalAnalysis", "ctx:SystemAnalysis", "la:LogicalArchitecture", "pa:PhysicalArchitecture", "epbs:EPBSArchitecture"] {
        assert!(capella.contains(kind), "{kind}");
    }
}
