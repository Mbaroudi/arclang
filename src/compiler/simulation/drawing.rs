//! The Modelica drawing of a model: what OpenModelica knows of a class of
//! the physics (its components, their icons and placements, the
//! connections and their lines) as the JSON of `getModelInstance`, for
//! VS Code to draw it, on the desktop and in the browser alike. The same
//! files as a simulation are loaded: the generated package, the Modelica
//! Standard Library and the hand-written sources of the simulated cases.

use super::tool::{self, ToolConfig};
use super::{is_simulated, spec_of};
use crate::compiler::modelica::{self, Export};
use crate::compiler::CompilationResult;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A class worth drawing: the model of a simulated case, or the system of a configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawable {
    pub class: String,
    pub label: String,
    pub kind: &'static str,
}

/// The sources of every simulated case, once each, as written (relative to the model).
fn sources(result: &CompilationResult) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for test in result.ast.test_cases.iter().filter(|t| is_simulated(t)) {
        if let Some(Ok(spec)) = spec_of(&test.attributes) {
            for source in spec.sources {
                if !all.contains(&source) {
                    all.push(source);
                }
            }
        }
    }
    all
}

/// What there is to draw: each simulated case's model, then each configuration's system.
pub fn drawables(result: &CompilationResult, export: &Export) -> Vec<Drawable> {
    let mut found: Vec<Drawable> = Vec::new();
    for test in result.ast.test_cases.iter().filter(|t| is_simulated(t)) {
        let Some(Ok(spec)) = spec_of(&test.attributes) else { continue };
        if !found.iter().any(|d| d.class == spec.model) {
            found.push(Drawable { class: spec.model.clone(), label: test.name.clone(), kind: "scenario" });
        }
    }
    for (configuration, system) in &export.systems {
        if found.iter().any(|d| &d.class == system) {
            continue;
        }
        let name = result.ast.configurations.iter().find(|c| &c.id == configuration).map(|c| c.name.clone()).unwrap_or_else(|| configuration.clone());
        let label = if name.is_empty() { "the system".to_string() } else { format!("configuration {name}") };
        found.push(Drawable { class: system.clone(), label, kind: "system" });
    }
    found
}

/// `{package, classes: [{class, label, kind}]}`: what `--drawings` prints.
pub fn drawables_json(result: &CompilationResult) -> Result<Value, String> {
    let export = modelica::export(&result.ast)?;
    let classes: Vec<Value> = drawables(result, &export).into_iter().map(|d| json!({ "class": d.class, "label": d.label, "kind": d.kind })).collect();
    Ok(json!({ "package": export.package, "classes": classes }))
}

/// The package as `arclang export -f modelica` writes it: placed and wired
/// (the layout file beside the model when there is one), for the drawing to
/// show what OMEdit shows. A simulation exports without placement.
fn placed_export(result: &CompilationResult, model_path: &Path) -> Result<Export, String> {
    use crate::compiler::diagram::layout_file::{sidecar_path, LayoutFile};
    let layout = std::fs::read_to_string(sidecar_path(model_path)).ok().and_then(|text| LayoutFile::parse(&text).ok());
    modelica::export_with(&result.ast, &modelica::Options { placement: true, layout })
}

/// A Modelica class name, nothing else: it is written into the script.
fn is_class_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 300
        && name.split('.').all(|part| {
            let mut chars = part.chars();
            chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// The top-level packages a drawing may name: the generated one, the sources' and the Standard Library.
fn known_roots(export: &Export, result: &CompilationResult) -> Vec<String> {
    let mut roots = vec![export.package.clone(), "Modelica".to_string()];
    for d in drawables(result, export) {
        if let Some(root) = d.class.split('.').next() {
            if !roots.iter().any(|r| r == root) {
                roots.push(root.to_string());
            }
        }
    }
    roots
}

fn work_dir() -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("arclang-drawing-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join("sources")).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

fn script(export: &Export, staged: &[String], class: &str) -> String {
    let msl = modelica::MSL_VERSION;
    let mut lines = vec![
        format!("if not loadModel(Modelica, {{\"{msl}\"}}) then installPackage(Modelica, \"{msl}+maint.om\", exactMatch=true); end if;"),
        format!("print(\"@@msl \" + String(loadModel(Modelica, {{\"{msl}\"}})) + \"\\n\");"),
        "print(getErrorString());".to_string(),
    ];
    for file in std::iter::once(&export.file_name).chain(staged) {
        lines.push(format!("print(\"@@load {file} \" + String(loadFile(\"{file}\")) + \"\\n\");"));
        lines.push("print(getErrorString());".to_string());
    }
    lines.push(format!("writeFile(\"instance.json\", getModelInstance({class}, prettyPrint=false));"));
    lines.push("print(getErrorString());".to_string());
    lines.join("\n") + "\n"
}

/// OpenModelica's instance of `class` (the JSON of `getModelInstance`), with the model's Modelica loaded.
pub fn instance(result: &CompilationResult, model_path: &Path, class: &str, config: &ToolConfig) -> Result<String, String> {
    if !is_class_name(class) {
        return Err(format!("'{class}' is not a Modelica class name"));
    }
    let export = placed_export(result, model_path)?;
    let roots = known_roots(&export, result);
    let root = class.split('.').next().unwrap_or_default();
    if !roots.iter().any(|r| r == root) {
        return Err(format!("'{class}' is not a class of this model's Modelica (its packages: {})", roots.join(", ")));
    }
    let tool = tool::locate(config)?;
    let base = super::cache::model_dir(model_path);
    let dir = work_dir()?;
    let staged = (|| -> Result<Vec<String>, String> {
        std::fs::write(dir.join(&export.file_name), &export.text).map_err(|e| e.to_string())?;
        let mut staged = Vec::new();
        for (index, source) in sources(result).iter().enumerate() {
            let path = base.join(source);
            let content = std::fs::read(&path).map_err(|e| format!("source '{source}' cannot be read ({}): {e}", path.display()))?;
            let stem: String = Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or("source").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            let name = format!("sources/{index}_{stem}.mo");
            std::fs::write(dir.join(&name), content).map_err(|e| e.to_string())?;
            staged.push(name);
        }
        Ok(staged)
    })();
    let staged = match staged {
        Ok(staged) => staged,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(error);
        }
    };
    let _ = std::fs::write(dir.join("drawing.mos"), script(&export, &staged, class));
    let run = tool::run(&tool, &dir, "drawing.mos", config.timeout);
    let json = std::fs::read_to_string(dir.join("instance.json")).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let run = run?;
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "null" || serde_json::from_str::<Value>(trimmed).is_err() {
        let tail: String = run.output.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("@@")).take(12).collect::<Vec<_>>().join("\n");
        return Err(format!("OpenModelica gave no instance of '{class}'{}", if tail.is_empty() { String::new() } else { format!(":\n{tail}") }));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_class_name_reaches_the_script() {
        for good in ["CoolingPhysics.SummerDay", "Modelica.Blocks.Sources.Step", "_A.b1"] {
            assert!(is_class_name(good), "{good}");
        }
        for bad in ["", "A..B", "1A", "A.B\"); system(\"rm", "A B", "A.B;", ".A"] {
            assert!(!is_class_name(bad), "{bad}");
        }
    }

    #[test]
    fn the_drawables_are_the_cases_models_then_the_configurations_systems() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/datacenter/ai-liquid-cooling/ai_liquid_cooling.arc");
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&path).expect("compiles");
        let listed = drawables_json(&result).expect("exports");
        let classes = listed["classes"].as_array().unwrap();
        assert_eq!(classes[0]["class"], "CoolingPhysics.SummerDay");
        assert_eq!(classes[0]["kind"], "scenario");
        assert!(classes.iter().any(|c| c["kind"] == "system" && c["class"].as_str().unwrap().contains(".Systems.")), "{listed}");
        let names: Vec<&str> = classes.iter().map(|c| c["class"].as_str().unwrap()).collect();
        let mut unique = names.clone();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "each class once");
    }

    #[test]
    fn a_class_outside_the_models_packages_is_refused_before_any_tool() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/datacenter/ai-liquid-cooling/ai_liquid_cooling.arc");
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&path).expect("compiles");
        let config = ToolConfig { omc: Some("/nonexistent/omc".into()), ..ToolConfig::default() };
        let error = instance(&result, &path, "Other.Package.Model", &config).unwrap_err();
        assert!(error.contains("not a class of this model's Modelica"), "{error}");
    }
}
