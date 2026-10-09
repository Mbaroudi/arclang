//! The OMG Systems Modeling API (REST binding) over compiled ArcLang
//! models: projects, branches, commits, elements, relationships, roots and
//! ad-hoc queries.
//!
//! What it is, precisely:
//! - The resource model, paths, JSON envelopes (`@id`, `@type`), cursor
//!   pagination (`page[size]`, `page[after]`, `page[before]`, `Link` header)
//!   and query constraints follow the specification.
//! - Two vocabularies are served from the same commits ([`Vocabulary`]):
//!   the ArcLang metamodel (`spec/METAMODEL.md`, `GET /api/metamodel`), and
//!   the KerML / SysML v2 metaclasses, read back from the model's SysML v2
//!   export (`sysml_view`).
//! - A project is one model on the branch `main`. Served as a single
//!   snapshot, its commit is content-addressed (same model → same commit
//!   id). Served WITH HISTORY (`arclang serve --history`), every git commit
//!   that touched the model file is a commit of the API, the uncommitted
//!   working tree is the head when it differs, and
//!   `GET .../commits/{id}/changes` lists the elements added, modified or
//!   deleted since the previous commit — a diff by identity, not by line.
//! - Access (`access`): open and read-only by default; users with a bearer
//!   token and a read or write role can be declared, and writes need both
//!   a user who may write and an explicit opt-in. A write (`write`,
//!   `operations`) is one git commit of the model file.
//! - Creating projects or branches is not supported.

use crate::compiler::elements::{attributes_json, ElementGraph};
use crate::compiler::identity::element_uuid;
use axum::{
    extract::{OriginalUri, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

const BRANCH_NAME: &str = "main";

/// The vocabulary a client reads a model in. Both are served side by side,
/// under two base paths, from the same commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vocabulary {
    /// `@type` is a kind of the ArcLang metamodel (`spec/METAMODEL.md`).
    ArcLang,
    /// `@type` is a KerML / SysML v2 metaclass (`PartUsage`, `ActionDefinition`...).
    SysML,
}

/// The records of one commit in one vocabulary.
#[derive(Default)]
pub struct View {
    /// Elements first, then relationships — the order of `GET .../elements`.
    records: Vec<Value>,
    index: HashMap<String, usize>,
    roots: Vec<usize>,
    /// (record index, source uuid, target uuid) of every relationship.
    ends: Vec<(usize, String, String)>,
}

impl View {
    fn new(records: Vec<Value>, roots: Vec<usize>, ends: Vec<(usize, String, String)>) -> View {
        let index = records
            .iter()
            .enumerate()
            .filter_map(|(position, record)| record["@id"].as_str().map(|uuid| (uuid.to_string(), position)))
            .collect();
        View { records, index, roots, ends }
    }

    fn record(&self, identity: &str) -> Option<&Value> {
        self.index.get(identity).map(|&position| &self.records[position])
    }
}

/// One compiled state of a model: a commit of the API.
pub struct Snapshot {
    pub id: String,
    pub description: String,
    pub created: String,
    /// The model in the ArcLang vocabulary.
    view: View,
    /// The same model in the SysML v2 vocabulary, or why it is unavailable.
    sysml: Result<View, String>,
    /// SHA-256 of the rendered records: equal digests = equal content.
    pub digest: String,
    pub unresolved: Vec<String>,
    pub warnings: Vec<String>,
    /// Git commit this snapshot was compiled from, when served with history.
    pub git_commit: Option<String>,
    /// The API user who made this commit, when a named user did.
    pub user: Option<String>,
    /// Why this revision could not be compiled (it then has no elements).
    pub compile_error: Option<String>,
}

fn reference(uuid: &str) -> Value {
    json!({ "@id": uuid })
}

fn references(uuids: &[String]) -> Value {
    Value::Array(uuids.iter().map(|uuid| reference(uuid)).collect())
}

fn timestamp(created: DateTime<Utc>) -> String {
    created.to_rfc3339_opts(SecondsFormat::Secs, true)
}

impl Snapshot {
    /// Render an element graph. The snapshot id is content-addressed until
    /// a history assigns it a git-based one.
    pub fn from_graph(graph: &ElementGraph, warnings: Vec<String>, created: DateTime<Utc>) -> Snapshot {
        let names: HashMap<&str, (&str, Option<&str>)> = graph
            .elements
            .iter()
            .map(|e| (e.uuid.as_str(), (e.name.as_str(), e.owner.as_deref())))
            .collect();
        let qualified = |uuid: &str| -> String {
            let mut segments = Vec::new();
            let mut cursor = Some(uuid);
            while let Some(current) = cursor {
                let Some((name, owner)) = names.get(current) else { break };
                segments.push(*name);
                cursor = *owner;
            }
            segments.reverse();
            segments.join("::")
        };
        let mut owned_elements: HashMap<&str, Vec<String>> = HashMap::new();
        for element in &graph.elements {
            if let Some(owner) = &element.owner {
                owned_elements.entry(owner.as_str()).or_default().push(element.uuid.clone());
            }
        }
        let mut owned_relationships: HashMap<&str, Vec<String>> = HashMap::new();
        for relationship in &graph.relationships {
            owned_relationships.entry(relationship.source.as_str()).or_default().push(relationship.uuid.clone());
        }

        let mut records = Vec::with_capacity(graph.elements.len() + graph.relationships.len());
        let mut roots = Vec::new();
        for element in &graph.elements {
            let mut object = Map::new();
            object.insert("@id".into(), json!(element.uuid));
            object.insert("@type".into(), json!(element.kind));
            object.insert("elementId".into(), json!(element.uuid));
            object.insert("name".into(), json!(element.name));
            object.insert("declaredName".into(), json!(element.name));
            object.insert("shortName".into(), json!(element.id));
            object.insert("declaredShortName".into(), json!(element.id));
            object.insert("qualifiedName".into(), json!(qualified(&element.uuid)));
            object.insert("owner".into(), element.owner.as_deref().map(reference).unwrap_or(Value::Null));
            object.insert(
                "ownedElement".into(),
                references(owned_elements.get(element.uuid.as_str()).map(Vec::as_slice).unwrap_or(&[])),
            );
            object.insert(
                "ownedRelationship".into(),
                references(owned_relationships.get(element.uuid.as_str()).map(Vec::as_slice).unwrap_or(&[])),
            );
            object.insert("attributes".into(), attributes_json(&element.attributes));
            for (key, value) in &element.extra {
                object.insert((*key).into(), value.clone());
            }
            if element.owner.is_none() {
                roots.push(records.len());
            }
            records.push(Value::Object(object));
        }
        let mut ends = Vec::with_capacity(graph.relationships.len());
        for relationship in &graph.relationships {
            let mut object = Map::new();
            object.insert("@id".into(), json!(relationship.uuid));
            object.insert("@type".into(), json!(relationship.kind));
            object.insert("elementId".into(), json!(relationship.uuid));
            object.insert("name".into(), json!(relationship.name));
            object.insert("declaredName".into(), json!(relationship.name));
            object.insert("owner".into(), reference(&relationship.source));
            object.insert("owningRelatedElement".into(), reference(&relationship.source));
            object.insert("source".into(), json!([reference(&relationship.source)]));
            object.insert("target".into(), json!([reference(&relationship.target)]));
            object.insert(
                "relatedElement".into(),
                json!([reference(&relationship.source), reference(&relationship.target)]),
            );
            object.insert("attributes".into(), attributes_json(&relationship.attributes));
            for (key, value) in &relationship.extra {
                object.insert((*key).into(), value.clone());
            }
            ends.push((records.len(), relationship.source.clone(), relationship.target.clone()));
            records.push(Value::Object(object));
        }

        let mut hasher = Sha256::new();
        for record in &records {
            hasher.update(record.to_string().as_bytes());
            hasher.update(b"\n");
        }
        let digest: String = hasher.finalize().iter().map(|byte| format!("{:02x}", byte)).collect();
        let name = graph.elements.first().map(|root| root.name.as_str()).unwrap_or("model");
        Snapshot {
            id: element_uuid("commit", &digest),
            description: format!(
                "Compiled snapshot of '{}': {} elements, {} relationships",
                name,
                records.len() - ends.len(),
                ends.len()
            ),
            created: timestamp(created),
            view: View::new(records, roots, ends),
            sysml: Err("the SysML v2 view is built from a compilation, not from a bare element graph".to_string()),
            digest,
            unresolved: graph.unresolved.clone(),
            warnings,
            git_commit: None,
            user: None,
            compile_error: None,
        }
    }

    /// One compiled state of a model, in both vocabularies.
    pub fn from_compilation(result: &crate::CompilationResult, created: DateTime<Utc>) -> Snapshot {
        let graph = crate::compiler::elements::build(&result.ast, &result.semantic_model);
        let mut snapshot = Snapshot::from_graph(&graph, result.warnings.clone(), created);
        snapshot.sysml = sysml_view::build(result, &snapshot.view);
        snapshot
    }

    /// The records of this commit in `vocabulary`.
    fn view(&self, vocabulary: Vocabulary) -> Result<&View, Response> {
        match (vocabulary, &self.sysml) {
            (Vocabulary::ArcLang, _) => Ok(&self.view),
            (Vocabulary::SysML, Ok(view)) => Ok(view),
            (Vocabulary::SysML, Err(reason)) => Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("the SysML v2 view of this commit is unavailable: {}", reason),
            )),
        }
    }

    /// A revision that does not compile: an honest, empty commit.
    fn failed(reason: String, created: DateTime<Utc>) -> Snapshot {
        Snapshot {
            id: element_uuid("commit", &format!("error|{}", reason)),
            description: "This revision does not compile".to_string(),
            created: timestamp(created),
            view: View::default(),
            sysml: Ok(View::default()),
            digest: String::new(),
            unresolved: Vec::new(),
            warnings: Vec::new(),
            git_commit: None,
            user: None,
            compile_error: Some(reason),
        }
    }
}

/// One model as the API serves it: its commits, oldest first. The last one
/// is the head of `main`.
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created: String,
    pub branch_id: String,
    pub commits: Vec<Snapshot>,
    /// The model file this project is compiled from, when there is one.
    pub source: Option<PathBuf>,
    /// Stable key of the file in its git repository, when served with history.
    history_key: Option<String>,
}

impl Project {
    fn single(name: &str, description: Option<String>, snapshot: Snapshot) -> Project {
        let id = element_uuid("project", name);
        Project {
            branch_id: element_uuid("branch", &format!("{}|{}", id, BRANCH_NAME)),
            id,
            name: name.to_string(),
            description,
            created: snapshot.created.clone(),
            commits: vec![snapshot],
            source: None,
            history_key: None,
        }
    }

    pub fn from_compilation(result: &crate::CompilationResult, created: DateTime<Utc>) -> Project {
        let graph = crate::compiler::elements::build(&result.ast, &result.semantic_model);
        let root = &graph.elements[0];
        let description = root.attributes.get("description").and_then(|v| v.as_string()).map(str::to_string);
        Project::single(&root.name, description, Snapshot::from_compilation(result, created))
    }

    /// Name (and therefore identity) of the project. A model without a
    /// `model Name { }` header is named after its file.
    fn renamed(mut self, name: &str) -> Project {
        self.id = element_uuid("project", name);
        self.branch_id = element_uuid("branch", &format!("{}|{}", self.id, BRANCH_NAME));
        self.name = name.to_string();
        self
    }

    /// The current state of the model.
    pub fn head(&self) -> &Snapshot {
        self.commits.last().expect("a project always has at least one commit")
    }

    fn position(&self, commit_id: &str) -> Option<usize> {
        self.commits.iter().position(|commit| commit.id == commit_id)
    }

    fn project_json(&self) -> Value {
        json!({
            "@id": self.id,
            "@type": "Project",
            "name": self.name,
            "description": self.description,
            "defaultBranch": reference(&self.branch_id),
            "created": self.created,
        })
    }

    fn branch_json(&self) -> Value {
        json!({
            "@id": self.branch_id,
            "@type": "Branch",
            "name": BRANCH_NAME,
            "owningProject": reference(&self.id),
            "head": reference(&self.head().id),
            "referencedCommit": reference(&self.head().id),
            "created": self.created,
        })
    }

    fn commit_json(&self, position: usize) -> Value {
        let commit = &self.commits[position];
        let previous: Vec<Value> = position.checked_sub(1).map(|p| reference(&self.commits[p].id)).into_iter().collect();
        json!({
            "@id": commit.id,
            "@type": "Commit",
            "description": commit.description,
            "owningProject": reference(&self.id),
            "previousCommit": previous,
            "created": commit.created,
            // ArcLang extensions: nothing the compiler knows is hidden.
            "arclang:contentDigest": commit.digest,
            "arclang:gitCommit": commit.git_commit,
            "arclang:user": commit.user,
            "arclang:compileError": commit.compile_error,
            "arclang:warnings": commit.warnings,
            "arclang:unresolvedRelationships": commit.unresolved,
        })
    }

    /// What changed in commit `position` relative to the one before it, by
    /// element identity: added and modified records carry their payload, a
    /// deleted record has a null payload. The first commit adds everything.
    fn changes(&self, position: usize, vocabulary: Vocabulary) -> Result<Vec<Value>, Response> {
        let commit = &self.commits[position];
        let view = commit.view(vocabulary)?;
        let previous = match position.checked_sub(1) {
            Some(before) => Some(self.commits[before].view(vocabulary)?),
            None => None,
        };
        let version = |identity: &str, kind: &str, payload: Value| {
            json!({
                "@id": element_uuid("version", &format!("{}|{}", commit.id, identity)),
                "@type": "DataVersion",
                "identity": { "@id": identity, "@type": "DataIdentity" },
                "payload": payload,
                "arclang:changeKind": kind,
            })
        };
        let mut changes = Vec::new();
        for record in &view.records {
            let identity = record["@id"].as_str().unwrap_or_default();
            match previous.and_then(|p| p.record(identity)) {
                None => changes.push(version(identity, "added", record.clone())),
                Some(before) if before != record => changes.push(version(identity, "modified", record.clone())),
                Some(_) => {}
            }
        }
        if let Some(previous) = previous {
            for record in &previous.records {
                let identity = record["@id"].as_str().unwrap_or_default();
                if !view.index.contains_key(identity) {
                    changes.push(version(identity, "deleted", Value::Null));
                }
            }
        }
        Ok(changes)
    }
}

/// The set of models a server instance serves.
#[derive(Default)]
pub struct Workspace {
    pub projects: Vec<Project>,
    pub access: Access,
}

/// The workspace as the server shares it between requests.
pub type SharedWorkspace = Arc<RwLock<Workspace>>;

fn read(workspace: &SharedWorkspace) -> RwLockReadGuard<'_, Workspace> {
    // A panic while writing leaves the last consistent state in place:
    // commits are appended only once everything else has succeeded.
    workspace.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write(workspace: &SharedWorkspace) -> RwLockWriteGuard<'_, Workspace> {
    workspace.write().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Workspace {
    fn add(&mut self, project: Project) -> Result<&Project, String> {
        if self.projects.iter().any(|existing| existing.id == project.id) {
            return Err(format!("two models are named '{}': project names must be unique in a workspace", project.name));
        }
        self.projects.push(project);
        Ok(self.projects.last().expect("just pushed"))
    }

    /// Compile a model given as text and serve it.
    pub fn add_source(&mut self, source: &str, created: DateTime<Utc>) -> Result<&Project, String> {
        let result = crate::Compiler::new(crate::CompilerConfig::default())
            .compile_string(source)
            .map_err(|e| e.to_string())?;
        self.add(Project::from_compilation(&result, created))
    }

    /// Compile a model file (imports resolved) and serve its current state.
    pub fn add_file(&mut self, path: &FsPath) -> Result<&Project, String> {
        let project = compile_file(path)?;
        self.add(project)
    }

    /// Serve a model file with its git history: one commit per git commit
    /// that touched it (at most `depth`, the most recent ones), plus the
    /// working tree as head when it differs from the last git commit.
    pub fn add_file_with_history(&mut self, path: &FsPath, depth: usize) -> Result<&Project, String> {
        let mut project = compile_file(path)?;
        let working_tree = project.commits.pop().expect("compile_file yields one snapshot");
        let (key, history) = history::snapshots(path, depth)?;
        project.commits = history;
        project.history_key = Some(key);
        // The working tree is a commit of its own only when it holds
        // something git does not: same content = same state.
        let same_as_last = project.commits.last().map_or(false, |last| last.digest == working_tree.digest);
        if !same_as_last {
            let mut working_tree = working_tree;
            working_tree.description = format!("Working tree (uncommitted): {}", working_tree.description);
            project.commits.push(working_tree);
        }
        if let Some(first) = project.commits.first() {
            project.created = first.created.clone();
        }
        self.add(project)
    }

    fn project(&self, id: &str) -> Result<&Project, Response> {
        self.projects
            .iter()
            .find(|project| project.id == id)
            .ok_or_else(|| not_found(format!("project '{}' not found", id)))
    }

    /// The project and the position of one of its commits.
    fn commit(&self, project_id: &str, commit_id: &str) -> Result<(&Project, usize), Response> {
        let project = self.project(project_id)?;
        match project.position(commit_id) {
            Some(position) => Ok((project, position)),
            None => Err(not_found(format!("commit '{}' not found in project '{}'", commit_id, project_id))),
        }
    }

    fn snapshot(&self, project_id: &str, commit_id: &str) -> Result<&Snapshot, Response> {
        self.commit(project_id, commit_id).map(|(project, position)| &project.commits[position])
    }
}

fn error(status: StatusCode, description: String) -> Response {
    (status, Json(json!({ "@type": "Error", "description": description }))).into_response()
}

fn not_found(description: String) -> Response {
    error(StatusCode::NOT_FOUND, description)
}

fn bad_request(description: String) -> Response {
    error(StatusCode::BAD_REQUEST, description)
}

fn compile_file(path: &FsPath) -> Result<Project, String> {
    let result = crate::Compiler::new(crate::CompilerConfig::default())
        .compile_file(path)
        .map_err(|e| format!("{}: {}", path.display(), e))?;
    let created = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map(DateTime::<Utc>::from)
        .map_err(|e| format!("{}: {}", path.display(), e))?;
    let mut project = Project::from_compilation(&result, created);
    project.source = Some(path.to_path_buf());
    Ok(match (&result.semantic_model.name, path.file_stem().and_then(|stem| stem.to_str())) {
        (None, Some(stem)) => project.renamed(stem),
        _ => project,
    })
}

mod access;
mod handlers;
mod history;
mod operations;
mod sysml_view;
mod write;

pub use access::{Access, Principal, Role, MIN_TOKEN_LENGTH};
pub use handlers::{authenticate, router};
