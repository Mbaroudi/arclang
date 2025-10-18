use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::requirements_management::*;

pub struct JamaConnector {
    client: Client,
    config: JamaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamaConfig {
    pub instance_url: String,
    pub project_id: i64,
    pub auth: RMAuthentication,
}

#[derive(Debug, Serialize, Deserialize)]
struct JamaItem {
    id: i64,
    #[serde(rename = "documentKey")]
    document_key: String,
    #[serde(rename = "globalId")]
    global_id: String,
    project: i64,
    #[serde(rename = "itemType")]
    item_type: i64,
    #[serde(rename = "childItemType")]
    child_item_type: Option<i64>,
    location: JamaLocation,
    fields: HashMap<String, serde_json::Value>,
    #[serde(rename = "createdDate")]
    created_date: String,
    #[serde(rename = "modifiedDate")]
    modified_date: String,
    #[serde(rename = "createdBy")]
    created_by: i64,
    #[serde(rename = "modifiedBy")]
    modified_by: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JamaLocation {
    parent: Option<i64>,
    #[serde(rename = "sortOrder")]
    sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JamaRelationship {
    id: i64,
    #[serde(rename = "fromItem")]
    from_item: i64,
    #[serde(rename = "toItem")]
    to_item: i64,
    #[serde(rename = "relationshipType")]
    relationship_type: i64,
    #[serde(rename = "relationshipTypeName")]
    relationship_type_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct JamaCreateItem {
    project: i64,
    #[serde(rename = "itemType")]
    item_type: i64,
    #[serde(rename = "childItemType")]
    child_item_type: Option<i64>,
    location: JamaCreateLocation,
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JamaCreateLocation {
    parent: i64,
}

#[derive(Debug, Serialize)]
struct JamaUpdateItem {
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JamaCreateRelationship {
    #[serde(rename = "fromItem")]
    from_item: i64,
    #[serde(rename = "toItem")]
    to_item: i64,
    #[serde(rename = "relationshipType")]
    relationship_type: i64,
}

#[derive(Debug, Deserialize)]
struct JamaItemsResponse {
    data: Vec<JamaItem>,
    meta: JamaMeta,
}

#[derive(Debug, Deserialize)]
struct JamaMeta {
    #[serde(rename = "pageInfo")]
    page_info: JamaPageInfo,
}

#[derive(Debug, Deserialize)]
struct JamaPageInfo {
    #[serde(rename = "startIndex")]
    start_index: i64,
    #[serde(rename = "resultCount")]
    result_count: i64,
    #[serde(rename = "totalResults")]
    total_results: i64,
}

#[derive(Debug, Deserialize)]
struct JamaRelationshipsResponse {
    data: Vec<JamaRelationship>,
}

#[derive(Debug, Deserialize)]
struct JamaCreateResponse {
    meta: JamaCreateMeta,
}

#[derive(Debug, Deserialize)]
struct JamaCreateMeta {
    location: String,
}

impl JamaConnector {
    pub fn new(config: JamaConfig) -> Self {
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
        format!("{}/rest/v1{}", self.config.instance_url, path)
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
            _ => Err(RMError::AuthenticationError(
                "Only BasicAuth and APIToken supported for Jama".to_string()
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
    
    fn convert_to_requirement(&self, item: JamaItem) -> Requirement {
        let title = item.fields.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let text = item.fields.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let status = item.fields.get("status")
            .and_then(|v| v.as_str())
            .map(|s| self.map_status(s))
            .unwrap_or(RequirementStatus::Draft);
        
        let priority = item.fields.get("priority")
            .and_then(|v| v.as_str())
            .map(|p| self.map_priority(p))
            .unwrap_or(RequirementPriority::Medium);
        
        let mut custom_attrs = HashMap::new();
        for (key, value) in item.fields {
            if !["name", "description", "status", "priority"].contains(&key.as_str()) {
                custom_attrs.insert(key, self.convert_json_value(value));
            }
        }
        
        Requirement {
            id: item.id.to_string(),
            external_id: Some(item.document_key),
            title,
            text,
            requirement_type: RequirementType::System,
            status,
            priority,
            rationale: None,
            acceptance_criteria: None,
            verification_method: None,
            verification_status: None,
            compliance: Vec::new(),
            custom_attributes: custom_attrs,
            parent_id: item.location.parent.map(|p| p.to_string()),
            children_ids: Vec::new(),
            created_at: chrono::DateTime::parse_from_rfc3339(&item.created_date)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            modified_at: chrono::DateTime::parse_from_rfc3339(&item.modified_date)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            created_by: item.created_by.to_string(),
            modified_by: item.modified_by.to_string(),
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
    
    fn map_status(&self, status: &str) -> RequirementStatus {
        match status.to_lowercase().as_str() {
            "draft" => RequirementStatus::Draft,
            "in review" | "review" => RequirementStatus::UnderReview,
            "approved" => RequirementStatus::Approved,
            "rejected" => RequirementStatus::Rejected,
            "obsolete" | "deprecated" => RequirementStatus::Obsolete,
            "implemented" | "complete" => RequirementStatus::Implemented,
            _ => RequirementStatus::Draft,
        }
    }
    
    fn map_priority(&self, priority: &str) -> RequirementPriority {
        match priority.to_lowercase().as_str() {
            "critical" | "urgent" => RequirementPriority::Critical,
            "high" => RequirementPriority::High,
            "low" => RequirementPriority::Low,
            _ => RequirementPriority::Medium,
        }
    }
    
    fn map_relationship_type(&self, type_name: &str) -> TraceLinkType {
        match type_name.to_lowercase().as_str() {
            "satisfies" | "satisfy" => TraceLinkType::Satisfies,
            "derives" | "derived from" => TraceLinkType::DerivedFrom,
            "refines" => TraceLinkType::Refines,
            "verifies" | "verified by" => TraceLinkType::VerifiedBy,
            "implements" => TraceLinkType::Implements,
            "depends on" => TraceLinkType::DependsOn,
            _ => TraceLinkType::Traces,
        }
    }
    
    async fn fetch_all_items(&self, start_index: i64) -> Result<Vec<JamaItem>, RMError> {
        let path = format!(
            "/items?project={}&startAt={}&maxResults=50",
            self.config.project_id,
            start_index
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch items: {}", response.status())
            ));
        }
        
        let items_response: JamaItemsResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let mut all_items = items_response.data;
        
        let total = items_response.meta.page_info.total_results;
        let fetched = start_index + items_response.meta.page_info.result_count;
        
        if fetched < total {
            let mut next_items = self.fetch_all_items(fetched).await?;
            all_items.append(&mut next_items);
        }
        
        Ok(all_items)
    }
}

#[async_trait]
impl RequirementsConnector for JamaConnector {
    fn name(&self) -> &str {
        "Jama Connect"
    }
    
    async fn connect(&mut self, _config: &RMConfig) -> Result<(), RMError> {
        let path = format!("/projects/{}", self.config.project_id);
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
        let all_items = self.fetch_all_items(0).await?;
        
        let requirements: HashMap<String, Requirement> = all_items
            .into_iter()
            .map(|item| {
                let req = self.convert_to_requirement(item);
                (req.id.clone(), req)
            })
            .collect();
        
        let mut trace_links = Vec::new();
        for (req_id, _) in &requirements {
            let links_path = format!("/items/{}/downstreamrelationships", req_id);
            
            if let Ok(links_response) = self.get_with_auth(&links_path).await {
                if links_response.status().is_success() {
                    if let Ok(rels_response) = links_response.json::<JamaRelationshipsResponse>().await {
                        for rel in rels_response.data {
                            trace_links.push(TraceLink {
                                id: rel.id.to_string(),
                                source_id: rel.from_item.to_string(),
                                target_id: rel.to_item.to_string(),
                                link_type: self.map_relationship_type(
                                    &rel.relationship_type_name.unwrap_or_else(|| "traces".to_string())
                                ),
                                rationale: None,
                                created_at: chrono::Utc::now(),
                                created_by: "jama".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(RMBaseline {
            timestamp: chrono::Utc::now(),
            system: "Jama".to_string(),
            project: self.config.project_id.to_string(),
            modules: Vec::new(),
            requirements,
            trace_links,
            metadata: RMMetadata {
                system_version: "8.77".to_string(),
                baseline_name: "ArcLang Sync".to_string(),
                created_by: "arclang".to_string(),
                description: None,
            },
        })
    }
    
    async fn fetch_requirement(&self, req_id: &str) -> Result<Requirement, RMError> {
        let path = format!("/items/{}", req_id);
        
        let response = self.get_with_auth(&path).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RMError::RequirementNotFound(req_id.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch requirement: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct SingleItemResponse {
            data: JamaItem,
        }
        
        let item_response: SingleItemResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(self.convert_to_requirement(item_response.data))
    }
    
    async fn fetch_module(&self, module_id: &str) -> Result<RequirementModule, RMError> {
        let path = format!("/items/{}", module_id);
        
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
        struct SingleItemResponse {
            data: JamaItem,
        }
        
        let item_response: SingleItemResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let item = item_response.data;
        
        Ok(RequirementModule {
            id: item.id.to_string(),
            name: item.fields.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string(),
            description: item.fields.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            parent_id: item.location.parent.map(|p| p.to_string()),
            requirements: Vec::new(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        })
    }
    
    async fn create_requirement(&self, req: &Requirement) -> Result<String, RMError> {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), serde_json::json!(req.title));
        fields.insert("description".to_string(), serde_json::json!(req.text));
        fields.insert("status".to_string(), serde_json::json!(format!("{:?}", req.status)));
        fields.insert("priority".to_string(), serde_json::json!(format!("{:?}", req.priority)));
        
        for (key, value) in &req.custom_attributes {
            fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let parent_id = req.parent_id.as_ref()
            .and_then(|p| p.parse::<i64>().ok())
            .unwrap_or(self.config.project_id);
        
        let create_item = JamaCreateItem {
            project: self.config.project_id,
            item_type: 101,
            child_item_type: None,
            location: JamaCreateLocation {
                parent: parent_id,
            },
            fields,
        };
        
        let path = "/items";
        
        let response = self.post_with_auth(path, &create_item).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create requirement: {}", response.status())
            ));
        }
        
        let create_response: JamaCreateResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let item_id = create_response.meta.location
            .rsplit('/')
            .next()
            .unwrap_or("0")
            .to_string();
        
        Ok(item_id)
    }
    
    async fn update_requirement(&self, req_id: &str, changes: &RequirementChanges) -> Result<(), RMError> {
        let mut fields = HashMap::new();
        
        if let Some(title) = &changes.title {
            fields.insert("name".to_string(), serde_json::json!(title));
        }
        
        if let Some(text) = &changes.text {
            fields.insert("description".to_string(), serde_json::json!(text));
        }
        
        if let Some(status) = &changes.status {
            fields.insert("status".to_string(), serde_json::json!(format!("{:?}", status)));
        }
        
        if let Some(priority) = &changes.priority {
            fields.insert("priority".to_string(), serde_json::json!(format!("{:?}", priority)));
        }
        
        for (key, value) in &changes.custom_attributes {
            fields.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let update_item = JamaUpdateItem { fields };
        
        let path = format!("/items/{}", req_id);
        
        let response = self.put_with_auth(&path, &update_item).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to update requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn delete_requirement(&self, req_id: &str) -> Result<(), RMError> {
        let path = format!("/items/{}", req_id);
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn create_trace_link(&self, link: &TraceLink) -> Result<String, RMError> {
        let from_item = link.source_id.parse::<i64>()
            .map_err(|_| RMError::ValidationError("Invalid source ID".to_string()))?;
        
        let to_item = link.target_id.parse::<i64>()
            .map_err(|_| RMError::ValidationError("Invalid target ID".to_string()))?;
        
        let create_rel = JamaCreateRelationship {
            from_item,
            to_item,
            relationship_type: 1,
        };
        
        let path = "/relationships";
        
        let response = self.post_with_auth(path, &create_rel).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create trace link: {}", response.status())
            ));
        }
        
        let create_response: JamaCreateResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let rel_id = create_response.meta.location
            .rsplit('/')
            .next()
            .unwrap_or("0")
            .to_string();
        
        Ok(rel_id)
    }
    
    async fn delete_trace_link(&self, link_id: &str) -> Result<(), RMError> {
        let path = format!("/relationships/{}", link_id);
        
        let response = self.delete_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to delete trace link: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn query_requirements(&self, filter: &RequirementFilter) -> Result<Vec<Requirement>, RMError> {
        let mut path = format!("/items?project={}", self.config.project_id);
        
        if let Some(text) = &filter.text_contains {
            path.push_str(&format!("&contains={}", urlencoding::encode(text)));
        }
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to query requirements: {}", response.status())
            ));
        }
        
        let items_response: JamaItemsResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let mut requirements: Vec<Requirement> = items_response.data.into_iter()
            .map(|item| self.convert_to_requirement(item))
            .collect();
        
        if let Some(status_filter) = &filter.status {
            requirements.retain(|r| &r.status == status_filter);
        }
        
        if let Some(priority_filter) = &filter.priority {
            requirements.retain(|r| &r.priority == priority_filter);
        }
        
        Ok(requirements)
    }
    
    async fn generate_traceability_matrix(&self, _from: &str, _to: &str) -> Result<TraceabilityMatrix, RMError> {
        Err(RMError::APIError("Traceability matrix generation not directly supported in Jama API".to_string()))
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
        
        let mut gaps = Vec::new();
        for (req_id, req) in &baseline.requirements {
            if !baseline.trace_links.iter().any(|link| link.source_id == *req_id) {
                gaps.push(CoverageGap {
                    requirement_id: req_id.clone(),
                    gap_type: GapType::NoTraceToDesign,
                    severity: GapSeverity::Medium,
                    description: format!("Requirement '{}' has no trace links", req.title),
                });
            }
        }
        
        Ok(CoverageReport {
            total_requirements,
            requirements_with_traces,
            requirements_verified,
            requirements_implemented,
            coverage_by_type: HashMap::new(),
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
    async fn test_jama_connection() {
        let config = JamaConfig {
            instance_url: "https://company.jamacloud.com".to_string(),
            project_id: 123,
            auth: RMAuthentication::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
        };
        
        let connector = JamaConnector::new(config);
        assert_eq!(connector.name(), "Jama Connect");
    }
}
