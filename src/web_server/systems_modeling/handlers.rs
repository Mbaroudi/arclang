//! HTTP handlers of the Systems Modeling API: pagination, resources, queries.

use super::*;

/// Cursor pagination of the specification: `page[size]` items after
/// `page[after]` or before `page[before]` (cursors are `@id` values), with
/// `Link` headers to the next and previous pages.
fn paginate(items: Vec<&Value>, params: &HashMap<String, String>, uri: &OriginalUri) -> Response {
    let size = match params.get("page[size]") {
        None => None,
        Some(raw) => match raw.parse::<usize>() {
            Ok(size) if size > 0 => Some(size),
            _ => return bad_request(format!("page[size] must be a positive integer, got '{}'", raw)),
        },
    };
    let position = |cursor: &str| items.iter().position(|item| item["@id"].as_str() == Some(cursor));
    let (start, end) = match (params.get("page[after]"), params.get("page[before]")) {
        (Some(_), Some(_)) => return bad_request("page[after] and page[before] are mutually exclusive".to_string()),
        (Some(after), None) => match position(after) {
            Some(index) => (index + 1, size.map_or(items.len(), |size| (index + 1 + size).min(items.len()))),
            None => return bad_request(format!("page[after]: unknown cursor '{}'", after)),
        },
        (None, Some(before)) => match position(before) {
            Some(index) => (size.map_or(0, |size| index.saturating_sub(size)), index),
            None => return bad_request(format!("page[before]: unknown cursor '{}'", before)),
        },
        (None, None) => (0, size.map_or(items.len(), |size| size.min(items.len()))),
    };
    let page: Vec<Value> = items[start..end].iter().map(|item| (*item).clone()).collect();

    let mut links = Vec::new();
    if let Some(size) = size {
        let path = uri.0.path();
        if end < items.len() {
            if let Some(last) = page.last().and_then(|item| item["@id"].as_str()) {
                links.push(format!("<{}?page[after]={}&page[size]={}>; rel=\"next\"", path, last, size));
            }
        }
        if start > 0 {
            if let Some(first) = page.first().and_then(|item| item["@id"].as_str()) {
                links.push(format!("<{}?page[before]={}&page[size]={}>; rel=\"prev\"", path, first, size));
            }
        }
    }
    let mut response = Json(Value::Array(page)).into_response();
    if !links.is_empty() {
        if let Ok(value) = HeaderValue::from_str(&links.join(", ")) {
            response.headers_mut().insert(header::LINK, value);
        }
    }
    response
}

type Params = Query<HashMap<String, String>>;

/// What a router serves: a workspace, read in one vocabulary.
#[derive(Clone)]
pub struct Api {
    pub(super) workspace: SharedWorkspace,
    pub(super) vocabulary: Vocabulary,
}

type Shared = State<Api>;

fn view_of<'w>(workspace: &'w Workspace, project_id: &str, commit_id: &str, vocabulary: Vocabulary) -> Result<&'w View, Response> {
    workspace.snapshot(project_id, commit_id)?.view(vocabulary)
}

/// Bearer-token authentication of every API request, when the workspace
/// has users. `/health` stays open: it says nothing about the models. The
/// authenticated user travels with the request as a [`Principal`].
pub async fn authenticate(
    State(workspace): State<SharedWorkspace>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let access = read(&workspace).access.clone();
    if !access.requires_token() || request.uri().path() == "/health" {
        return next.run(request).await;
    }
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .and_then(|token| access.authenticate(token.trim()));
    match presented {
        Some(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        None => {
            let mut response = error(
                StatusCode::UNAUTHORIZED,
                "this API requires a bearer token (Authorization: Bearer <token>)".to_string(),
            );
            response.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
            response
        }
    }
}

async fn list_projects(State(api): Shared, Query(params): Params, uri: OriginalUri) -> Response {
    let workspace = read(&api.workspace);
    let projects: Vec<Value> = workspace.projects.iter().map(Project::project_json).collect();
    paginate(projects.iter().collect(), &params, &uri)
}

async fn get_project(State(api): Shared, Path(project_id): Path<String>) -> Response {
    let workspace = read(&api.workspace);
    match workspace.project(&project_id) {
        Ok(project) => Json(project.project_json()).into_response(),
        Err(response) => response,
    }
}

async fn list_branches(State(api): Shared, Path(project_id): Path<String>) -> Response {
    let workspace = read(&api.workspace);
    match workspace.project(&project_id) {
        Ok(project) => Json(json!([project.branch_json()])).into_response(),
        Err(response) => response,
    }
}

async fn get_branch(State(api): Shared, Path((project_id, branch_id)): Path<(String, String)>) -> Response {
    let workspace = read(&api.workspace);
    match workspace.project(&project_id) {
        Ok(project) if project.branch_id == branch_id => Json(project.branch_json()).into_response(),
        Ok(_) => not_found(format!("branch '{}' not found in project '{}'", branch_id, project_id)),
        Err(response) => response,
    }
}

/// Commits, newest first (the head of `main` comes first).
async fn list_commits(State(api): Shared, Path(project_id): Path<String>, Query(params): Params, uri: OriginalUri) -> Response {
    let workspace = read(&api.workspace);
    match workspace.project(&project_id) {
        Ok(project) => {
            let commits: Vec<Value> = (0..project.commits.len()).rev().map(|position| project.commit_json(position)).collect();
            paginate(commits.iter().collect(), &params, &uri)
        }
        Err(response) => response,
    }
}

async fn get_commit(State(api): Shared, Path((project_id, commit_id)): Path<(String, String)>) -> Response {
    let workspace = read(&api.workspace);
    match workspace.commit(&project_id, &commit_id) {
        Ok((project, position)) => Json(project.commit_json(position)).into_response(),
        Err(response) => response,
    }
}

/// Elements added, modified or deleted by a commit, by identity.
async fn list_changes(
    State(api): Shared,
    Path((project_id, commit_id)): Path<(String, String)>,
    Query(params): Params,
    uri: OriginalUri,
) -> Response {
    let workspace = read(&api.workspace);
    match workspace.commit(&project_id, &commit_id) {
        Ok((project, position)) => {
            match project.changes(position, api.vocabulary) {
                Ok(changes) => paginate(changes.iter().collect(), &params, &uri),
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

async fn list_elements(
    State(api): Shared,
    Path((project_id, commit_id)): Path<(String, String)>,
    Query(params): Params,
    uri: OriginalUri,
) -> Response {
    let workspace = read(&api.workspace);
    match view_of(&workspace, &project_id, &commit_id, api.vocabulary) {
        Ok(snapshot) => paginate(snapshot.records.iter().collect(), &params, &uri),
        Err(response) => response,
    }
}

async fn list_roots(
    State(api): Shared,
    Path((project_id, commit_id)): Path<(String, String)>,
    Query(params): Params,
    uri: OriginalUri,
) -> Response {
    let workspace = read(&api.workspace);
    match view_of(&workspace, &project_id, &commit_id, api.vocabulary) {
        Ok(snapshot) => paginate(snapshot.roots.iter().map(|&index| &snapshot.records[index]).collect(), &params, &uri),
        Err(response) => response,
    }
}

async fn get_element(
    State(api): Shared,
    Path((project_id, commit_id, element_id)): Path<(String, String, String)>,
) -> Response {
    let workspace = read(&api.workspace);
    let project = match view_of(&workspace, &project_id, &commit_id, api.vocabulary) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };
    match project.index.get(&element_id) {
        Some(&index) => Json(project.records[index].clone()).into_response(),
        None => not_found(format!("element '{}' not found in commit '{}'", element_id, commit_id)),
    }
}

async fn get_relationships(
    State(api): Shared,
    Path((project_id, commit_id, element_id)): Path<(String, String, String)>,
    Query(params): Params,
    uri: OriginalUri,
) -> Response {
    let workspace = read(&api.workspace);
    let project = match view_of(&workspace, &project_id, &commit_id, api.vocabulary) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };
    if !project.index.contains_key(&element_id) {
        return not_found(format!("element '{}' not found in commit '{}'", element_id, commit_id));
    }
    let direction = params.get("direction").map(String::as_str).unwrap_or("both");
    if !matches!(direction, "in" | "out" | "both") {
        return bad_request(format!("direction must be in, out or both, got '{}'", direction));
    }
    let related: Vec<&Value> = project
        .ends
        .iter()
        .filter(|(_, source, target)| match direction {
            "out" => source == &element_id,
            "in" => target == &element_id,
            _ => source == &element_id || target == &element_id,
        })
        .map(|(index, _, _)| &project.records[*index])
        .collect();
    paginate(related, &params, &uri)
}

/// Value of a (possibly dotted) property of a record: `@type`, `name`,
/// `attributes.latency.canonicalValue`, `source.@id` (first array item).
fn property<'v>(record: &'v Value, path: &str) -> Option<&'v Value> {
    let mut current = record;
    for segment in path.split('.') {
        current = match current {
            Value::Array(items) => items.first()?.get(segment)?,
            other => other.get(segment)?,
        };
    }
    Some(current)
}

fn compare(operator: &str, actual: Option<&Value>, expected: &Value) -> Result<bool, String> {
    let ordering = || -> Option<std::cmp::Ordering> {
        match (actual?, expected) {
            (Value::Number(a), Value::Number(b)) => a.as_f64()?.partial_cmp(&b.as_f64()?),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            _ => None,
        }
    };
    Ok(match operator {
        "=" => actual == Some(expected),
        "<" => ordering() == Some(std::cmp::Ordering::Less),
        ">" => ordering() == Some(std::cmp::Ordering::Greater),
        "<=" => matches!(ordering(), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
        ">=" => matches!(ordering(), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
        // Membership: the property is one of the listed values.
        "in" => expected.as_array().map_or(false, |values| actual.map_or(false, |a| values.contains(a))),
        // The spec's instanceOf tests a metatype; kinds here have no
        // subtyping, so it is equality on `@type`.
        "instanceOf" => actual == Some(expected),
        other => return Err(format!("unsupported operator '{}' (supported: =, <, >, <=, >=, in, instanceOf)", other)),
    })
}

fn satisfies(record: &Value, constraint: &Value) -> Result<bool, String> {
    match constraint["@type"].as_str() {
        Some("PrimitiveConstraint") => {
            let path = constraint["property"].as_str().ok_or("PrimitiveConstraint needs a string 'property'")?;
            let operator = constraint["operator"].as_str().ok_or("PrimitiveConstraint needs an 'operator'")?;
            let expected = constraint.get("value").ok_or("PrimitiveConstraint needs a 'value'")?;
            let path = if operator == "instanceOf" { "@type" } else { path };
            let result = compare(operator, property(record, path), expected)?;
            Ok(result != constraint["inverse"].as_bool().unwrap_or(false))
        }
        Some("CompositeConstraint") => {
            let parts = constraint["constraint"].as_array().ok_or("CompositeConstraint needs a 'constraint' array")?;
            match constraint["operator"].as_str() {
                Some("and") => {
                    for part in parts {
                        if !satisfies(record, part)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                Some("or") => {
                    for part in parts {
                        if satisfies(record, part)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                other => Err(format!("CompositeConstraint operator must be 'and' or 'or', got {:?}", other)),
            }
        }
        other => Err(format!("constraint '@type' must be PrimitiveConstraint or CompositeConstraint, got {:?}", other)),
    }
}

/// `POST /projects/{id}/query-results` — evaluate an ad-hoc Query:
/// `{"@type":"Query","select":[...],"where":{constraint}}`.
async fn query_results(
    State(api): Shared,
    Path(project_id): Path<String>,
    Query(params): Params,
    Json(query): Json<Value>,
) -> Response {
    let workspace = read(&api.workspace);
    // The query runs on the commit named by `commitId`, the head by default.
    let project = match params.get("commitId") {
        Some(commit_id) => view_of(&workspace, &project_id, commit_id, api.vocabulary),
        None => workspace.project(&project_id).and_then(|project| project.head().view(api.vocabulary)),
    };
    let project = match project {
        Ok(view) => view,
        Err(response) => return response,
    };
    let select: Option<Vec<&str>> = match query.get("select") {
        None | Some(Value::Null) => None,
        Some(Value::Array(items)) => match items.iter().map(Value::as_str).collect::<Option<Vec<_>>>() {
            Some(names) if !names.is_empty() => Some(names),
            Some(_) => None,
            None => return bad_request("'select' must be an array of property names".to_string()),
        },
        Some(_) => return bad_request("'select' must be an array of property names".to_string()),
    };
    let mut results = Vec::new();
    for record in &project.records {
        let keep = match query.get("where") {
            None | Some(Value::Null) => true,
            Some(constraint) => match satisfies(record, constraint) {
                Ok(keep) => keep,
                Err(reason) => return bad_request(reason),
            },
        };
        if !keep {
            continue;
        }
        results.push(match &select {
            None => record.clone(),
            Some(names) => {
                let mut projected = Map::new();
                projected.insert("@id".into(), record["@id"].clone());
                for name in names {
                    projected.insert((*name).to_string(), property(record, name).cloned().unwrap_or(Value::Null));
                }
                Value::Object(projected)
            }
        });
    }
    Json(Value::Array(results)).into_response()
}

/// The Systems Modeling API router, to be nested under a base path.
pub fn router(workspace: SharedWorkspace, vocabulary: Vocabulary) -> Router {
    Router::new()
        .route("/projects", get(list_projects))
        .route("/projects/:project_id", get(get_project))
        .route("/projects/:project_id/branches", get(list_branches))
        .route("/projects/:project_id/branches/:branch_id", get(get_branch))
        .route("/projects/:project_id/commits", get(list_commits).post(write::create_commit))
        .route("/projects/:project_id/commits/:commit_id", get(get_commit))
        .route("/projects/:project_id/commits/:commit_id/elements", get(list_elements))
        .route("/projects/:project_id/commits/:commit_id/elements/:element_id", get(get_element))
        .route("/projects/:project_id/commits/:commit_id/elements/:element_id/relationships", get(get_relationships))
        .route("/projects/:project_id/commits/:commit_id/roots", get(list_roots))
        .route("/projects/:project_id/commits/:commit_id/changes", get(list_changes))
        .route("/projects/:project_id/query-results", post(query_results))
        .with_state(Api { workspace, vocabulary })
}
