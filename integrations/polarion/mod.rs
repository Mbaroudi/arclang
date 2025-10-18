use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::requirements_management::*;

pub struct PolarionConnector {
    client: Client,
    config: PolarionConfig,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarionConfig {
    pub server_url: String,
    pub project_id: String,
    pub auth: RMAuthentication,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarionWorkItem {
    id: String,
    #[serde(rename = "type")]
    work_item_type: String,
    title: String,
    description: Option<PolarionText>,
    status: String,
    priority: String,
    #[serde(rename = "customFields")]
    custom_fields: HashMap<String, serde_json::Value>,
    author: String,
    created: String,
    updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarionText {
    #[serde(rename = "type")]
    content_type: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarionLinkedWorkItem {
    #[serde(rename = "workItemId")]
    work_item_id: String,
    role: String,
}

#[derive(Debug, Serialize)]
struct PolarionCreateWorkItem {
    project: String,
    #[serde(rename = "type")]
    work_item_type: String,
    title: String,
    description: PolarionText,
    #[serde(rename = "customFields")]
    custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct PolarionUpdateWorkItem {
    title: Option<String>,
    description: Option<PolarionText>,
    status: Option<String>,
    priority: Option<String>,
    #[serde(rename = "customFields")]
    custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct PolarionLinkWorkItems {
    #[serde(rename = "workItemId")]
    work_item_id: String,
    #[serde(rename = "linkedWorkItemId")]
    linked_work_item_id: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct PolarionWorkItemsResponse {
    #[serde(rename = "workItems")]
    work_items: Vec<PolarionWorkItem>,
}

impl PolarionConnector {
    pub fn new(config: PolarionConfig) -> Self {
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
            session_id: None,
        }
    }
    
    async fn authenticate(&mut self) -> Result<(), RMError> {
        let login_url = format!("{}/polarion/rest/v1/auth/login", self.config.server_url);
        
        let credentials = match &self.config.auth {
            RMAuthentication::BasicAuth { username, password } => {
                serde_json::json!({
                    "login": username,
                    "password": password
                })
            }
            RMAuthentication::PAT { personal_access_token } => {
                self.session_id = Some(personal_access_token.clone());
                return Ok(());
            }
            _ => {
                return Err(RMError::AuthenticationError(
                    "Only BasicAuth and PAT supported for Polarion".to_string()
                ));
            }
        };
        
        let response = self.client
            .post(&login_url)
            .json(&credentials)
            .send()
            .await
            .map_err(|e| RMError::AuthenticationError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(RMError::AuthenticationError(
                format!("Polarion login failed: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }
        
        let login_response: LoginResponse = response.json().await
            .map_err(|e| RMError::AuthenticationError(e.to_string()))?;
        
        self.session_id = Some(login_response.token);
        
        Ok(())
    }
    
    fn build_url(&self, path: &str) -> String {
        format!("{}/polarion/rest/v1{}", self.config.server_url, path)
    }
    
    async fn get_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.get(&url);
        
        if let Some(token) = &self.session_id {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn post_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.post(&url).json(body);
        
        if let Some(token) = &self.session_id {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn patch_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.patch(&url).json(body);
        
        if let Some(token) = &self.session_id {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn delete_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.delete(&url);
        
        if let Some(token) = &self.session_id {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    fn convert_to_requirement(&self, work_item: PolarionWorkItem) -> Requirement {
        let text = work_item.description
            .as_ref()
            .map(|d| d.content.clone())
            .unwrap_or_default();
        
        let mut custom_attrs = HashMap::new();
        for (key, value) in work_item.custom_fields {
            custom_attrs.insert(key, self.convert_json_value(value));
        }
        
        Requirement {
            id: work_item.id.clone(),
            external_id: Some(work_item.id.clone()),
            title: work_item.title,
            text,
            requirement_type: self.map_work_item_type(&work_item.work_item_type),
            status: self.map_status(&work_item.status),
            priority: self.map_priority(&work_item.priority),
            rationale: None,
            acceptance_criteria: None,
            verification_method: None,
            verification_status: None,
            compliance: Vec::new(),
            custom_attributes: custom_attrs,
            parent_id: None,
            children_ids: Vec::new(),
            created_at: chrono::DateTime::parse_from_rfc3339(&work_item.created)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            modified_at: chrono::DateTime::parse_from_rfc3339(&work_item.updated)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            created_by: work_item.author.clone(),
            modified_by: work_item.author,
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
    
    fn map_work_item_type(&self, wi_type: &str) -> RequirementType {
        match wi_type.to_lowercase().as_str() {
            "requirement" => RequirementType::System,
            "functionalrequirement" => RequirementType::Functional,
            "performancerequirement" => RequirementType::Performance,
            "safetyrequirement" => RequirementType::Safety,
            "securityrequirement" => RequirementType::Security,
            _ => RequirementType::System,
        }
    }
    
    fn map_status(&self, status: &str) -> RequirementStatus {
        match status.to_lowercase().as_str() {
            "draft" => RequirementStatus::Draft,
            "open" | "inreview" => RequirementStatus::UnderReview,
            "approved" | "accepted" => RequirementStatus::Approved,
            "rejected" => RequirementStatus::Rejected,
            "obsolete" | "closed" => RequirementStatus::Obsolete,
            "implemented" | "done" => RequirementStatus::Implemented,
            _ => RequirementStatus::Draft,
        }
    }
    
    fn map_priority(&self, priority: &str) -> RequirementPriority {
        match priority.to_lowercase().as_str() {
            "urgent" | "critical" => RequirementPriority::Critical,
            "high" => RequirementPriority::High,
            "low" => RequirementPriority::Low,
            _ => RequirementPriority::Medium,
        }
    }
    
    fn map_to_polarion_status(&self, status: &RequirementStatus) -> &str {
        match status {
            RequirementStatus::Draft => "draft",
            RequirementStatus::UnderReview => "inReview",
            RequirementStatus::Approved => "approved",
            RequirementStatus::Rejected => "rejected",
            RequirementStatus::Obsolete => "obsolete",
            RequirementStatus::Implemented => "implemented",
        }
    }
    
    fn map_to_polarion_priority(&self, priority: &RequirementPriority) -> &str {
        match priority {
            RequirementPriority::Critical => "critical",
            RequirementPriority::High => "high",
            RequirementPriority::Medium => "medium",
            RequirementPriority::Low => "low",
        }
    }
    
    fn map_link_role(&self, link_type: &TraceLinkType) -> String {
        match link_type {
            TraceLinkType::Satisfies => "satisfies".to_string(),
            TraceLinkType::DerivedFrom => "parent".to_string(),
            TraceLinkType::Refines => "refines".to_string(),
            TraceLinkType::VerifiedBy => "verifies".to_string(),
            TraceLinkType::AllocatedTo => "relates_to".to_string(),
            TraceLinkType::Implements => "implements".to_string(),
            _ => "relates_to".to_string(),
        }
    }
    
    fn map_from_link_role(&self, role: &str) -> TraceLinkType {
        match role.to_lowercase().as_str() {
            "satisfies" => TraceLinkType::Satisfies,
            "parent" => TraceLinkType::DerivedFrom,
            "refines" => TraceLinkType::Refines,
            "verifies" => TraceLinkType::VerifiedBy,
            "implements" => TraceLinkType::Implements,
            _ => TraceLinkType::Traces,
        }
    }
}

#[async_trait]
impl RequirementsConnector for PolarionConnector {
    fn name(&self) -> &str {
        "Polarion ALM"
    }
    
    async fn connect(&mut self, _config: &RMConfig) -> Result<(), RMError> {
        self.authenticate().await?;
        
        let test_path = format!("/projects/{}", self.config.project_id);
        let response = self.get_with_auth(&test_path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::ConnectionError(
                format!("Failed to connect to project: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), RMError> {
        if self.session_id.is_some() {
            let logout_url = "/auth/logout";
            let _ = self.post_with_auth(logout_url, &serde_json::json!({})).await;
            self.session_id = None;
        }
        
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<RMBaseline, RMError> {
        let path = format!(
            "/projects/{}/workitems?query=type:requirement",
            self.config.project_id
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch baseline: {}", response.status())
            ));
        }
        
        let work_items_response: PolarionWorkItemsResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let requirements: HashMap<String, Requirement> = work_items_response.work_items
            .into_iter()
            .map(|wi| {
                let req = self.convert_to_requirement(wi);
                (req.id.clone(), req)
            })
            .collect();
        
        let mut trace_links = Vec::new();
        for (req_id, _) in &requirements {
            let links_path = format!(
                "/projects/{}/workitems/{}/linkedWorkItems",
                self.config.project_id,
                req_id
            );
            
            if let Ok(links_response) = self.get_with_auth(&links_path).await {
                if links_response.status().is_success() {
                    #[derive(Deserialize)]
                    struct LinkedItemsResponse {
                        #[serde(rename = "linkedWorkItems")]
                        linked_work_items: Vec<PolarionLinkedWorkItem>,
                    }
                    
                    if let Ok(links_data) = links_response.json::<LinkedItemsResponse>().await {
                        for link in links_data.linked_work_items {
                            trace_links.push(TraceLink {
                                id: format!("{}-{}", req_id, link.work_item_id),
                                source_id: req_id.clone(),
                                target_id: link.work_item_id,
                                link_type: self.map_from_link_role(&link.role),
                                rationale: None,
                                created_at: chrono::Utc::now(),
                                created_by: "polarion".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(RMBaseline {
            timestamp: chrono::Utc::now(),
            system: "Polarion".to_string(),
            project: self.config.project_id.clone(),
            modules: Vec::new(),
            requirements,
            trace_links,
            metadata: RMMetadata {
                system_version: "23.3".to_string(),
                baseline_name: "ArcLang Sync".to_string(),
                created_by: "arclang".to_string(),
                description: None,
            },
        })
    }
    
    async fn fetch_requirement(&self, req_id: &str) -> Result<Requirement, RMError> {
        let path = format!(
            "/projects/{}/workitems/{}",
            self.config.project_id,
            req_id
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RMError::RequirementNotFound(req_id.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch requirement: {}", response.status())
            ));
        }
        
        let work_item: PolarionWorkItem = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(self.convert_to_requirement(work_item))
    }
    
    async fn fetch_module(&self, module_id: &str) -> Result<RequirementModule, RMError> {
        let path = format!(
            "/projects/{}/documents/{}",
            self.config.project_id,
            module_id
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RMError::ModuleNotFound(module_id.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch module: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct DocumentResponse {
            id: String,
            title: String,
            description: Option<String>,
        }
        
        let doc: DocumentResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(RequirementModule {
            id: doc.id,
            name: doc.title,
            description: doc.description,
            parent_id: None,
            requirements: Vec::new(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        })
    }
    
    async fn create_requirement(&self, req: &Requirement) -> Result<String, RMError> {
        let mut custom_fields = HashMap::new();
        for (key, value) in &req.custom_attributes {
            custom_fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let create_wi = PolarionCreateWorkItem {
            project: self.config.project_id.clone(),
            work_item_type: "requirement".to_string(),
            title: req.title.clone(),
            description: PolarionText {
                content_type: "text/html".to_string(),
                content: req.text.clone(),
            },
            custom_fields,
        };
        
        let path = format!("/projects/{}/workitems", self.config.project_id);
        
        let response = self.post_with_auth(&path, &create_wi).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create requirement: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }
        
        let create_response: CreateResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(create_response.id)
    }
    
    async fn update_requirement(&self, req_id: &str, changes: &RequirementChanges) -> Result<(), RMError> {
        let mut custom_fields = HashMap::new();
        for (key, value) in &changes.custom_attributes {
            custom_fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let update_wi = PolarionUpdateWorkItem {
            title: changes.title.clone(),
            description: changes.text.as_ref().map(|text| PolarionText {
                content_type: "text/html".to_string(),
                content: text.clone(),
            }),
            status: changes.status.as_ref().map(|s| self.map_to_polarion_status(s).to_string()),
            priority: changes.priority.as_ref().map(|p| self.map_to_polarion_priority(p).to_string()),
            custom_fields,
        };
        
        let path = format!(
            "/projects/{}/workitems/{}",
            self.config.project_id,
            req_id
        );
        
        let response = self.patch_with_auth(&path, &update_wi).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to update requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn delete_requirement(&self, req_id: &str) -> Result<(), RMError> {
        let path = format!(
            "/projects/{}/workitems/{}",
            self.config.project_id,
            req_id
        );
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn create_trace_link(&self, link: &TraceLink) -> Result<String, RMError> {
        let link_wi = PolarionLinkWorkItems {
            work_item_id: link.source_id.clone(),
            linked_work_item_id: link.target_id.clone(),
            role: self.map_link_role(&link.link_type),
        };
        
        let path = format!(
            "/projects/{}/workitems/{}/linkedWorkItems",
            self.config.project_id,
            link.source_id
        );
        
        let response = self.post_with_auth(&path, &link_wi).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create trace link: {}", response.status())
            ));
        }
        
        Ok(format!("{}-{}", link.source_id, link.target_id))
    }
    
    async fn delete_trace_link(&self, link_id: &str) -> Result<(), RMError> {
        let parts: Vec<&str> = link_id.split('-').collect();
        if parts.len() < 2 {
            return Err(RMError::ValidationError("Invalid link ID format".to_string()));
        }
        
        let path = format!(
            "/projects/{}/workitems/{}/linkedWorkItems/{}",
            self.config.project_id,
            parts[0],
            parts[1]
        );
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete trace link: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn query_requirements(&self, filter: &RequirementFilter) -> Result<Vec<Requirement>, RMError> {
        let mut query_parts = vec!["type:requirement".to_string()];
        
        if let Some(status) = &filter.status {
            query_parts.push(format!("status:{}", self.map_to_polarion_status(status)));
        }
        
        if let Some(priority) = &filter.priority {
            query_parts.push(format!("priority:{}", self.map_to_polarion_priority(priority)));
        }
        
        if let Some(text) = &filter.text_contains {
            query_parts.push(format!("title:*{}*", text));
        }
        
        let query = query_parts.join(" AND ");
        
        let path = format!(
            "/projects/{}/workitems?query={}",
            self.config.project_id,
            urlencoding::encode(&query)
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to query requirements: {}", response.status())
            ));
        }
        
        let work_items_response: PolarionWorkItemsResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(work_items_response.work_items.into_iter()
            .map(|wi| self.convert_to_requirement(wi))
            .collect())
    }
    
    async fn generate_traceability_matrix(&self, from: &str, to: &str) -> Result<TraceabilityMatrix, RMError> {
        let path = format!(
            "/projects/{}/traceability?from={}&to={}",
            self.config.project_id,
            urlencoding::encode(from),
            urlencoding::encode(to)
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to generate traceability matrix: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct MatrixResponse {
            rows: Vec<String>,
            columns: Vec<String>,
            links: Vec<(String, String)>,
        }
        
        let matrix_response: MatrixResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let rows: Vec<MatrixRow> = matrix_response.rows.into_iter()
            .map(|id| MatrixRow {
                id: id.clone(),
                label: id,
            })
            .collect();
        
        let columns: Vec<MatrixColumn> = matrix_response.columns.into_iter()
            .map(|id| MatrixColumn {
                id: id.clone(),
                label: id,
            })
            .collect();
        
        let cells: Vec<MatrixCell> = matrix_response.links.into_iter()
            .map(|(row_id, col_id)| MatrixCell {
                row_id,
                column_id: col_id,
                has_link: true,
                link_type: Some(TraceLinkType::Traces),
            })
            .collect();
        
        let coverage_percentage = if rows.is_empty() {
            0.0
        } else {
            (cells.len() as f64 / rows.len() as f64) * 100.0
        };
        
        Ok(TraceabilityMatrix {
            source_type: from.to_string(),
            target_type: to.to_string(),
            rows,
            columns,
            cells,
            coverage_percentage,
        })
    }
    
    async fn get_coverage_report(&self) -> Result<CoverageReport, RMError> {
        let baseline = self.fetch_baseline().await?;
        
        let total_requirements = baseline.requirements.len();
        let requirements_with_traces = baseline.requirements.values()
            .filter(|req| {
                baseline.trace_links.iter().any(|link| link.source_id == req.id)
            })
            .count();
        
        let requirements_verified = baseline.requirements.values()
            .filter(|req| req.verification_status == Some(VerificationStatus::Passed))
            .count();
        
        let requirements_implemented = baseline.requirements.values()
            .filter(|req| req.status == RequirementStatus::Implemented)
            .count();
        
        let mut coverage_by_type = HashMap::new();
        for req_type in &[
            RequirementType::System,
            RequirementType::Functional,
            RequirementType::Performance,
            RequirementType::Safety,
        ] {
            let type_reqs: Vec<_> = baseline.requirements.values()
                .filter(|r| &r.requirement_type == req_type)
                .collect();
            
            let total = type_reqs.len();
            let with_traces = type_reqs.iter()
                .filter(|req| {
                    baseline.trace_links.iter().any(|link| link.source_id == req.id)
                })
                .count();
            
            let verified = type_reqs.iter()
                .filter(|req| req.verification_status == Some(VerificationStatus::Passed))
                .count();
            
            let coverage_percentage = if total > 0 {
                (with_traces as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            coverage_by_type.insert(req_type.clone(), CoverageStats {
                total,
                with_traces,
                verified,
                coverage_percentage,
            });
        }
        
        let mut gaps = Vec::new();
        for (req_id, req) in &baseline.requirements {
            if !baseline.trace_links.iter().any(|link| link.source_id == *req_id) {
                gaps.push(CoverageGap {
                    requirement_id: req_id.clone(),
                    gap_type: GapType::NoTraceToDesign,
                    severity: if req.requirement_type == RequirementType::Safety {
                        GapSeverity::Critical
                    } else {
                        GapSeverity::Medium
                    },
                    description: format!("Requirement '{}' has no trace to design", req.title),
                });
            }
            
            if req.verification_method.is_none() {
                gaps.push(CoverageGap {
                    requirement_id: req_id.clone(),
                    gap_type: GapType::NoVerification,
                    severity: GapSeverity::High,
                    description: format!("Requirement '{}' has no verification method", req.title),
                });
            }
        }
        
        Ok(CoverageReport {
            total_requirements,
            requirements_with_traces,
            requirements_verified,
            requirements_implemented,
            coverage_by_type,
            gaps,
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
    async fn test_polarion_connection() {
        let config = PolarionConfig {
            server_url: "https://polarion.company.com".to_string(),
            project_id: "afcs".to_string(),
            auth: RMAuthentication::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
        };
        
        let connector = PolarionConnector::new(config);
        assert_eq!(connector.name(), "Polarion ALM");
    }
}
