use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::requirements_management::*;

pub struct JiraConnector {
    client: Client,
    config: JiraConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub base_url: String,
    pub project_key: String,
    pub auth: RMAuthentication,
    pub issue_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraIssue {
    id: String,
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraFields {
    summary: String,
    description: Option<String>,
    issuetype: JiraIssueType,
    status: JiraStatus,
    priority: JiraPriority,
    created: String,
    updated: String,
    reporter: JiraUser,
    assignee: Option<JiraUser>,
    #[serde(flatten)]
    custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraPriority {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraUser {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraIssueLink {
    id: String,
    #[serde(rename = "type")]
    link_type: JiraIssueLinkType,
    #[serde(rename = "inwardIssue")]
    inward_issue: Option<JiraLinkedIssue>,
    #[serde(rename = "outwardIssue")]
    outward_issue: Option<JiraLinkedIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraIssueLinkType {
    name: String,
    inward: String,
    outward: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraLinkedIssue {
    key: String,
}

#[derive(Debug, Serialize)]
struct JiraCreateIssue {
    fields: JiraCreateFields,
}

#[derive(Debug, Serialize)]
struct JiraCreateFields {
    project: JiraProject,
    summary: String,
    description: String,
    issuetype: JiraIssueTypeInput,
    priority: Option<JiraPriorityInput>,
    #[serde(flatten)]
    custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JiraProject {
    key: String,
}

#[derive(Debug, Serialize)]
struct JiraIssueTypeInput {
    name: String,
}

#[derive(Debug, Serialize)]
struct JiraPriorityInput {
    name: String,
}

#[derive(Debug, Serialize)]
struct JiraUpdateIssue {
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JiraCreateIssueLink {
    #[serde(rename = "type")]
    link_type: JiraIssueLinkTypeInput,
    #[serde(rename = "inwardIssue")]
    inward_issue: JiraIssueRef,
    #[serde(rename = "outwardIssue")]
    outward_issue: JiraIssueRef,
}

#[derive(Debug, Serialize)]
struct JiraIssueLinkTypeInput {
    name: String,
}

#[derive(Debug, Serialize)]
struct JiraIssueRef {
    key: String,
}

#[derive(Debug, Deserialize)]
struct JiraSearchResponse {
    issues: Vec<JiraIssue>,
    total: i64,
    #[serde(rename = "startAt")]
    start_at: i64,
    #[serde(rename = "maxResults")]
    max_results: i64,
}

#[derive(Debug, Deserialize)]
struct JiraCreateResponse {
    id: String,
    key: String,
}

impl JiraConnector {
    pub fn new(config: JiraConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            config,
        }
    }
    
    fn build_url(&self, path: &str) -> String {
        format!("{}/rest/api/3{}", self.config.base_url, path)
    }
    
    fn get_auth_header(&self) -> Result<String, RMError> {
        match &self.config.auth {
            RMAuthentication::BasicAuth { username, password } => {
                let credentials = format!("{}:{}", username, password);
                Ok(format!("Basic {}", base64::encode(credentials)))
            }
            RMAuthentication::APIToken { token } => {
                Ok(format!("Bearer {}", token))
            }
            RMAuthentication::PAT { personal_access_token } => {
                Ok(format!("Bearer {}", personal_access_token))
            }
            _ => Err(RMError::AuthenticationError(
                "Unsupported authentication method for Jira".to_string()
            )),
        }
    }
    
    async fn get_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        let auth_header = self.get_auth_header()?;
        
        self.client
            .get(&url)
            .header(header::AUTHORIZATION, auth_header)
            .send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn post_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        let auth_header = self.get_auth_header()?;
        
        self.client
            .post(&url)
            .header(header::AUTHORIZATION, auth_header)
            .json(body)
            .send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn put_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        let auth_header = self.get_auth_header()?;
        
        self.client
            .put(&url)
            .header(header::AUTHORIZATION, auth_header)
            .json(body)
            .send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn delete_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        let auth_header = self.get_auth_header()?;
        
        self.client
            .delete(&url)
            .header(header::AUTHORIZATION, auth_header)
            .send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    fn convert_to_requirement(&self, issue: JiraIssue) -> Requirement {
        let text = issue.fields.description
            .clone()
            .unwrap_or_default();
        
        let mut custom_attrs = HashMap::new();
        for (key, value) in issue.fields.custom_fields {
            if key.starts_with("customfield_") {
                custom_attrs.insert(key, self.convert_json_value(value));
            }
        }
        
        Requirement {
            id: issue.key.clone(),
            external_id: Some(issue.id),
            title: issue.fields.summary,
            text,
            requirement_type: self.map_issue_type(&issue.fields.issuetype.name),
            status: self.map_status(&issue.fields.status.name),
            priority: self.map_priority(&issue.fields.priority.name),
            rationale: None,
            acceptance_criteria: None,
            verification_method: None,
            verification_status: None,
            compliance: Vec::new(),
            custom_attributes: custom_attrs,
            parent_id: None,
            children_ids: Vec::new(),
            created_at: chrono::DateTime::parse_from_rfc3339(&issue.fields.created)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            modified_at: chrono::DateTime::parse_from_rfc3339(&issue.fields.updated)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            created_by: issue.fields.reporter.display_name.clone(),
            modified_by: issue.fields.assignee
                .map(|a| a.display_name)
                .unwrap_or(issue.fields.reporter.display_name),
        }
    }
    
    fn convert_json_value(&self, value: serde_json::Value) -> AttributeValue {
        match value {
            serde_json::Value::String(s) => AttributeValue::String(s),
            serde_json::Value::Number(n) => AttributeValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => AttributeValue::Boolean(b),
            serde_json::Value::Array(arr) => {
                AttributeValue::List(
                    arr.into_iter()
                        .filter_map(|v| {
                            if let serde_json::Value::String(s) = v {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect()
                )
            }
            _ => AttributeValue::String(value.to_string()),
        }
    }
    
    fn map_issue_type(&self, issue_type: &str) -> RequirementType {
        match issue_type.to_lowercase().as_str() {
            "requirement" => RequirementType::System,
            "story" | "user story" => RequirementType::Functional,
            "epic" => RequirementType::Stakeholder,
            "task" => RequirementType::Functional,
            _ => RequirementType::System,
        }
    }
    
    fn map_status(&self, status: &str) -> RequirementStatus {
        match status.to_lowercase().as_str() {
            "open" | "to do" | "backlog" => RequirementStatus::Draft,
            "in progress" | "in review" => RequirementStatus::UnderReview,
            "done" | "closed" | "resolved" => RequirementStatus::Approved,
            "rejected" | "canceled" => RequirementStatus::Rejected,
            "obsolete" => RequirementStatus::Obsolete,
            _ => RequirementStatus::Draft,
        }
    }
    
    fn map_priority(&self, priority: &str) -> RequirementPriority {
        match priority.to_lowercase().as_str() {
            "highest" | "blocker" => RequirementPriority::Critical,
            "high" => RequirementPriority::High,
            "low" | "lowest" => RequirementPriority::Low,
            _ => RequirementPriority::Medium,
        }
    }
    
    fn map_to_jira_priority(&self, priority: &RequirementPriority) -> &str {
        match priority {
            RequirementPriority::Critical => "Highest",
            RequirementPriority::High => "High",
            RequirementPriority::Medium => "Medium",
            RequirementPriority::Low => "Low",
        }
    }
    
    fn map_link_type(&self, link_type_name: &str) -> TraceLinkType {
        match link_type_name.to_lowercase().as_str() {
            "relates to" => TraceLinkType::Traces,
            "blocks" | "is blocked by" => TraceLinkType::DependsOn,
            "duplicates" | "is duplicated by" => TraceLinkType::Refines,
            "implements" => TraceLinkType::Implements,
            _ => TraceLinkType::Traces,
        }
    }
    
    fn map_to_jira_link_type(&self, link_type: &TraceLinkType) -> &str {
        match link_type {
            TraceLinkType::Satisfies => "Relates",
            TraceLinkType::DerivedFrom => "Relates",
            TraceLinkType::Refines => "Relates",
            TraceLinkType::VerifiedBy => "Relates",
            TraceLinkType::Implements => "Implements",
            TraceLinkType::DependsOn => "Blocks",
            _ => "Relates",
        }
    }
    
    async fn search_issues(&self, jql: &str, start_at: i64) -> Result<Vec<JiraIssue>, RMError> {
        let path = format!(
            "/search?jql={}&startAt={}&maxResults=50",
            urlencoding::encode(jql),
            start_at
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to search issues: {}", response.status())
            ));
        }
        
        let search_response: JiraSearchResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let mut all_issues = search_response.issues;
        
        let fetched = start_at + search_response.max_results;
        if fetched < search_response.total {
            let mut next_issues = self.search_issues(jql, fetched).await?;
            all_issues.append(&mut next_issues);
        }
        
        Ok(all_issues)
    }
}

#[async_trait]
impl RequirementsConnector for JiraConnector {
    fn name(&self) -> &str {
        "Jira"
    }
    
    async fn connect(&mut self, _config: &RMConfig) -> Result<(), RMError> {
        let path = format!("/project/{}", self.config.project_key);
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::ConnectionError(
                format!("Failed to connect to project: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), RMError> {
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<RMBaseline, RMError> {
        let jql = format!(
            "project = {} AND type = '{}'",
            self.config.project_key,
            self.config.issue_type
        );
        
        let all_issues = self.search_issues(&jql, 0).await?;
        
        let requirements: HashMap<String, Requirement> = all_issues
            .into_iter()
            .map(|issue| {
                let req = self.convert_to_requirement(issue);
                (req.id.clone(), req)
            })
            .collect();
        
        let mut trace_links = Vec::new();
        for (req_key, _) in &requirements {
            let links_path = format!("/issue/{}", req_key);
            
            if let Ok(issue_response) = self.get_with_auth(&links_path).await {
                if issue_response.status().is_success() {
                    if let Ok(issue_data) = issue_response.json::<JiraIssue>().await {
                        if let Some(links_data) = issue_data.fields.custom_fields.get("issuelinks") {
                            if let Some(links_array) = links_data.as_array() {
                                for link_value in links_array {
                                    if let Ok(link) = serde_json::from_value::<JiraIssueLink>(link_value.clone()) {
                                        if let Some(outward) = link.outward_issue {
                                            trace_links.push(TraceLink {
                                                id: link.id.clone(),
                                                source_id: req_key.clone(),
                                                target_id: outward.key,
                                                link_type: self.map_link_type(&link.link_type.name),
                                                rationale: None,
                                                created_at: chrono::Utc::now(),
                                                created_by: "jira".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(RMBaseline {
            timestamp: chrono::Utc::now(),
            system: "Jira".to_string(),
            project: self.config.project_key.clone(),
            modules: Vec::new(),
            requirements,
            trace_links,
            metadata: RMMetadata {
                system_version: "9.x".to_string(),
                baseline_name: "ArcLang Sync".to_string(),
                created_by: "arclang".to_string(),
                description: None,
            },
        })
    }
    
    async fn fetch_requirement(&self, req_id: &str) -> Result<Requirement, RMError> {
        let path = format!("/issue/{}", req_id);
        
        let response = self.get_with_auth(&path).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RMError::RequirementNotFound(req_id.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch requirement: {}", response.status())
            ));
        }
        
        let issue: JiraIssue = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(self.convert_to_requirement(issue))
    }
    
    async fn fetch_module(&self, _module_id: &str) -> Result<RequirementModule, RMError> {
        Err(RMError::APIError("Modules not supported in Jira".to_string()))
    }
    
    async fn create_requirement(&self, req: &Requirement) -> Result<String, RMError> {
        let mut custom_fields = HashMap::new();
        for (key, value) in &req.custom_attributes {
            custom_fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let create_issue = JiraCreateIssue {
            fields: JiraCreateFields {
                project: JiraProject {
                    key: self.config.project_key.clone(),
                },
                summary: req.title.clone(),
                description: req.text.clone(),
                issuetype: JiraIssueTypeInput {
                    name: self.config.issue_type.clone(),
                },
                priority: Some(JiraPriorityInput {
                    name: self.map_to_jira_priority(&req.priority).to_string(),
                }),
                custom_fields,
            },
        };
        
        let path = "/issue";
        
        let response = self.post_with_auth(path, &create_issue).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create requirement: {}", response.status())
            ));
        }
        
        let create_response: JiraCreateResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(create_response.key)
    }
    
    async fn update_requirement(&self, req_id: &str, changes: &RequirementChanges) -> Result<(), RMError> {
        let mut fields = HashMap::new();
        
        if let Some(title) = &changes.title {
            fields.insert("summary".to_string(), serde_json::json!(title));
        }
        
        if let Some(text) = &changes.text {
            fields.insert("description".to_string(), serde_json::json!(text));
        }
        
        if let Some(priority) = &changes.priority {
            fields.insert("priority".to_string(), serde_json::json!({
                "name": self.map_to_jira_priority(priority)
            }));
        }
        
        for (key, value) in &changes.custom_attributes {
            fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let update_issue = JiraUpdateIssue { fields };
        
        let path = format!("/issue/{}", req_id);
        
        let response = self.put_with_auth(&path, &update_issue).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to update requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn delete_requirement(&self, req_id: &str) -> Result<(), RMError> {
        let path = format!("/issue/{}", req_id);
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn create_trace_link(&self, link: &TraceLink) -> Result<String, RMError> {
        let create_link = JiraCreateIssueLink {
            link_type: JiraIssueLinkTypeInput {
                name: self.map_to_jira_link_type(&link.link_type).to_string(),
            },
            inward_issue: JiraIssueRef {
                key: link.source_id.clone(),
            },
            outward_issue: JiraIssueRef {
                key: link.target_id.clone(),
            },
        };
        
        let path = "/issueLink";
        
        let response = self.post_with_auth(path, &create_link).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create trace link: {}", response.status())
            ));
        }
        
        Ok(format!("{}-{}", link.source_id, link.target_id))
    }
    
    async fn delete_trace_link(&self, link_id: &str) -> Result<(), RMError> {
        let path = format!("/issueLink/{}", link_id);
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete trace link: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn query_requirements(&self, filter: &RequirementFilter) -> Result<Vec<Requirement>, RMError> {
        let mut jql_parts = vec![
            format!("project = {}", self.config.project_key),
            format!("type = '{}'", self.config.issue_type),
        ];
        
        if let Some(status) = &filter.status {
            jql_parts.push(format!("status = '{:?}'", status));
        }
        
        if let Some(priority) = &filter.priority {
            jql_parts.push(format!("priority = '{}'", self.map_to_jira_priority(priority)));
        }
        
        if let Some(text) = &filter.text_contains {
            jql_parts.push(format!("text ~ \"{}\"", text));
        }
        
        let jql = jql_parts.join(" AND ");
        
        let all_issues = self.search_issues(&jql, 0).await?;
        
        Ok(all_issues.into_iter()
            .map(|issue| self.convert_to_requirement(issue))
            .collect())
    }
    
    async fn generate_traceability_matrix(&self, _from: &str, _to: &str) -> Result<TraceabilityMatrix, RMError> {
        Err(RMError::APIError("Traceability matrix not natively supported in Jira".to_string()))
    }
    
    async fn get_coverage_report(&self) -> Result<CoverageReport, RMError> {
        let baseline = self.fetch_baseline().await?;
        
        let total_requirements = baseline.requirements.len();
        let requirements_with_traces = baseline.requirements.values()
            .filter(|req| {
                baseline.trace_links.iter().any(|link| link.source_id == req.id)
            })
            .count();
        
        let requirements_verified = 0;
        let requirements_implemented = baseline.requirements.values()
            .filter(|req| req.status == RequirementStatus::Approved)
            .count();
        
        Ok(CoverageReport {
            total_requirements,
            requirements_with_traces,
            requirements_verified,
            requirements_implemented,
            coverage_by_type: HashMap::new(),
            gaps: Vec::new(),
        })
    }
    
    fn attribute_value_to_json(&self, value: &AttributeValue) -> serde_json::Value {
        match value {
            AttributeValue::String(s) => serde_json::json!(s),
            AttributeValue::Number(n) => serde_json::json!(n),
            AttributeValue::Boolean(b) => serde_json::json!(b),
            AttributeValue::Date(d) => serde_json::json!(d.to_rfc3339()),
            AttributeValue::List(l) => serde_json::json!(l),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_jira_connection() {
        let config = JiraConfig {
            base_url: "https://company.atlassian.net".to_string(),
            project_key: "AFCS".to_string(),
            auth: RMAuthentication::BasicAuth {
                username: "test@company.com".to_string(),
                password: "api_token".to_string(),
            },
            issue_type: "Requirement".to_string(),
        };
        
        let connector = JiraConnector::new(config);
        assert_eq!(connector.name(), "Jira");
    }
}
