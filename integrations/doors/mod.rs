use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::requirements_management::*;

pub struct DOORSConnector {
    client: Client,
    config: DOORSConfig,
    session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOORSConfig {
    pub server_url: String,
    pub database: String,
    pub project: String,
    pub module: String,
    pub auth: RMAuthentication,
}

#[derive(Debug, Serialize, Deserialize)]
struct DOORSModule {
    id: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "objectCount")]
    object_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct DOORSObject {
    id: String,
    #[serde(rename = "objectNumber")]
    object_number: String,
    #[serde(rename = "objectText")]
    object_text: String,
    #[serde(rename = "objectHeading")]
    object_heading: String,
    #[serde(rename = "objectType")]
    object_type: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DOORSLink {
    id: String,
    #[serde(rename = "sourceId")]
    source_id: String,
    #[serde(rename = "targetId")]
    target_id: String,
    #[serde(rename = "linkType")]
    link_type: String,
}

#[derive(Debug, Serialize)]
struct DOORSCreateObject {
    #[serde(rename = "objectText")]
    object_text: String,
    #[serde(rename = "objectHeading")]
    object_heading: String,
    #[serde(rename = "objectType")]
    object_type: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct DOORSUpdateObject {
    id: String,
    #[serde(rename = "objectText")]
    object_text: Option<String>,
    #[serde(rename = "objectHeading")]
    object_heading: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct DOORSCreateLink {
    #[serde(rename = "sourceId")]
    source_id: String,
    #[serde(rename = "targetId")]
    target_id: String,
    #[serde(rename = "linkType")]
    link_type: String,
}

impl DOORSConnector {
    pub fn new(config: DOORSConfig) -> Self {
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
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            config,
            session_token: None,
        }
    }
    
    async fn authenticate(&mut self) -> Result<(), RMError> {
        let login_url = format!("{}/dwa/api/login", self.config.server_url);
        
        let credentials = match &self.config.auth {
            RMAuthentication::BasicAuth { username, password } => {
                serde_json::json!({
                    "username": username,
                    "password": password,
                    "database": self.config.database
                })
            }
            _ => {
                return Err(RMError::AuthenticationError(
                    "Only BasicAuth supported for DOORS".to_string()
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
                format!("DOORS login failed: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }
        
        let login_response: LoginResponse = response.json().await
            .map_err(|e| RMError::AuthenticationError(e.to_string()))?;
        
        self.session_token = Some(login_response.token);
        
        Ok(())
    }
    
    fn build_url(&self, path: &str) -> String {
        format!("{}/dwa/api{}", self.config.server_url, path)
    }
    
    async fn get_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.get(&url);
        
        if let Some(token) = &self.session_token {
            req = req.header("X-DOORS-Token", token);
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn post_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.post(&url).json(body);
        
        if let Some(token) = &self.session_token {
            req = req.header("X-DOORS-Token", token);
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn put_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.put(&url).json(body);
        
        if let Some(token) = &self.session_token {
            req = req.header("X-DOORS-Token", token);
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    async fn delete_with_auth(&self, path: &str) -> Result<reqwest::Response, RMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.delete(&url);
        
        if let Some(token) = &self.session_token {
            req = req.header("X-DOORS-Token", token);
        }
        
        req.send()
            .await
            .map_err(|e| RMError::NetworkError(e.to_string()))
    }
    
    fn convert_to_requirement(&self, doors_obj: DOORSObject) -> Requirement {
        let mut custom_attrs = HashMap::new();
        
        for (key, value) in doors_obj.attributes {
            custom_attrs.insert(key, self.convert_json_value(value));
        }
        
        Requirement {
            id: doors_obj.id.clone(),
            external_id: Some(doors_obj.object_number),
            title: doors_obj.object_heading,
            text: doors_obj.object_text,
            requirement_type: RequirementType::System,
            status: RequirementStatus::Approved,
            priority: RequirementPriority::Medium,
            rationale: None,
            acceptance_criteria: None,
            verification_method: None,
            verification_status: None,
            compliance: Vec::new(),
            custom_attributes: custom_attrs,
            parent_id: None,
            children_ids: Vec::new(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            created_by: "doors".to_string(),
            modified_by: "doors".to_string(),
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
}

#[async_trait]
impl RequirementsConnector for DOORSConnector {
    fn name(&self) -> &str {
        "DOORS Classic"
    }
    
    async fn connect(&mut self, _config: &RMConfig) -> Result<(), RMError> {
        self.authenticate().await?;
        
        let test_path = format!(
            "/projects/{}/modules",
            self.config.project
        );
        
        let response = self.get_with_auth(&test_path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::ConnectionError(
                format!("Failed to connect: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), RMError> {
        if self.session_token.is_some() {
            let logout_url = "/logout";
            let _ = self.post_with_auth(logout_url, &serde_json::json!({})).await;
            self.session_token = None;
        }
        
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<RMBaseline, RMError> {
        let module_path = format!(
            "/projects/{}/modules/{}/objects",
            self.config.project,
            self.config.module
        );
        
        let response = self.get_with_auth(&module_path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to fetch baseline: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct ObjectsResponse {
            objects: Vec<DOORSObject>,
        }
        
        let objects_response: ObjectsResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        let requirements: HashMap<String, Requirement> = objects_response.objects
            .into_iter()
            .map(|obj| {
                let req = self.convert_to_requirement(obj);
                (req.id.clone(), req)
            })
            .collect();
        
        let links_path = format!(
            "/projects/{}/modules/{}/links",
            self.config.project,
            self.config.module
        );
        
        let links_response = self.get_with_auth(&links_path).await?;
        
        let trace_links = if links_response.status().is_success() {
            #[derive(Deserialize)]
            struct LinksResponse {
                links: Vec<DOORSLink>,
            }
            
            let links_data: LinksResponse = links_response.json().await
                .map_err(|e| RMError::SerializationError(e.to_string()))?;
            
            links_data.links.into_iter().map(|link| TraceLink {
                id: link.id,
                source_id: link.source_id,
                target_id: link.target_id,
                link_type: self.map_link_type(&link.link_type),
                rationale: None,
                created_at: chrono::Utc::now(),
                created_by: "doors".to_string(),
            }).collect()
        } else {
            Vec::new()
        };
        
        Ok(RMBaseline {
            timestamp: chrono::Utc::now(),
            system: "DOORS".to_string(),
            project: self.config.project.clone(),
            modules: vec![RequirementModule {
                id: self.config.module.clone(),
                name: self.config.module.clone(),
                description: None,
                parent_id: None,
                requirements: requirements.keys().cloned().collect(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
            }],
            requirements,
            trace_links,
            metadata: RMMetadata {
                system_version: "9.7".to_string(),
                baseline_name: "ArcLang Sync".to_string(),
                created_by: "arclang".to_string(),
                description: None,
            },
        })
    }
    
    async fn fetch_requirement(&self, req_id: &str) -> Result<Requirement, RMError> {
        let path = format!(
            "/projects/{}/modules/{}/objects/{}",
            self.config.project,
            self.config.module,
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
        
        let doors_obj: DOORSObject = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(self.convert_to_requirement(doors_obj))
    }
    
    async fn fetch_module(&self, module_id: &str) -> Result<RequirementModule, RMError> {
        let path = format!(
            "/projects/{}/modules/{}",
            self.config.project,
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
        
        let doors_module: DOORSModule = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(RequirementModule {
            id: doors_module.id,
            name: doors_module.name,
            description: doors_module.description,
            parent_id: None,
            requirements: Vec::new(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        })
    }
    
    async fn create_requirement(&self, req: &Requirement) -> Result<String, RMError> {
        let mut attributes = HashMap::new();
        
        for (key, value) in &req.custom_attributes {
            attributes.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let create_obj = DOORSCreateObject {
            object_text: req.text.clone(),
            object_heading: req.title.clone(),
            object_type: "Requirement".to_string(),
            attributes,
        };
        
        let path = format!(
            "/projects/{}/modules/{}/objects",
            self.config.project,
            self.config.module
        );
        
        let response = self.post_with_auth(&path, &create_obj).await?;
        
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
        let mut attributes = HashMap::new();
        
        for (key, value) in &changes.custom_attributes {
            attributes.insert(key.clone(), self.attribute_value_to_json(value));
        }
        
        let update_obj = DOORSUpdateObject {
            id: req_id.to_string(),
            object_text: changes.text.clone(),
            object_heading: changes.title.clone(),
            attributes,
        };
        
        let path = format!(
            "/projects/{}/modules/{}/objects/{}",
            self.config.project,
            self.config.module,
            req_id
        );
        
        let response = self.put_with_auth(&path, &update_obj).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to update requirement: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn delete_requirement(&self, req_id: &str) -> Result<(), RMError> {
        let path = format!(
            "/projects/{}/modules/{}/objects/{}",
            self.config.project,
            self.config.module,
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
        let create_link = DOORSCreateLink {
            source_id: link.source_id.clone(),
            target_id: link.target_id.clone(),
            link_type: self.map_to_doors_link_type(&link.link_type),
        };
        
        let path = format!(
            "/projects/{}/modules/{}/links",
            self.config.project,
            self.config.module
        );
        
        let response = self.post_with_auth(&path, &create_link).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to create trace link: {}", response.status())
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
    
    async fn delete_trace_link(&self, link_id: &str) -> Result<(), RMError> {
        let path = format!(
            "/projects/{}/modules/{}/links/{}",
            self.config.project,
            self.config.module,
            link_id
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
        let mut query_params = Vec::new();
        
        if let Some(text) = &filter.text_contains {
            query_params.push(format!("text={}", urlencoding::encode(text)));
        }
        
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };
        
        let path = format!(
            "/projects/{}/modules/{}/objects{}",
            self.config.project,
            self.config.module,
            query_string
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(RMError::APIError(
                format!("Failed to query requirements: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct QueryResponse {
            objects: Vec<DOORSObject>,
        }
        
        let query_response: QueryResponse = response.json().await
            .map_err(|e| RMError::SerializationError(e.to_string()))?;
        
        Ok(query_response.objects.into_iter()
            .map(|obj| self.convert_to_requirement(obj))
            .collect())
    }
    
    async fn generate_traceability_matrix(&self, _from: &str, _to: &str) -> Result<TraceabilityMatrix, RMError> {
        Err(RMError::APIError("Traceability matrix generation not supported in DOORS Classic".to_string()))
    }
    
    async fn get_coverage_report(&self) -> Result<CoverageReport, RMError> {
        let baseline = self.fetch_baseline().await?;
        
        let total_requirements = baseline.requirements.len();
        let requirements_with_traces = baseline.requirements.values()
            .filter(|req| {
                baseline.trace_links.iter().any(|link| link.source_id == req.id)
            })
            .count();
        
        Ok(CoverageReport {
            total_requirements,
            requirements_with_traces,
            requirements_verified: 0,
            requirements_implemented: 0,
            coverage_by_type: HashMap::new(),
            gaps: Vec::new(),
        })
    }
    
    fn map_link_type(&self, doors_type: &str) -> TraceLinkType {
        match doors_type.to_lowercase().as_str() {
            "satisfies" => TraceLinkType::Satisfies,
            "derivedfrom" => TraceLinkType::DerivedFrom,
            "refines" => TraceLinkType::Refines,
            "verifiedby" => TraceLinkType::VerifiedBy,
            _ => TraceLinkType::Traces,
        }
    }
    
    fn map_to_doors_link_type(&self, link_type: &TraceLinkType) -> String {
        match link_type {
            TraceLinkType::Satisfies => "Satisfies".to_string(),
            TraceLinkType::DerivedFrom => "DerivedFrom".to_string(),
            TraceLinkType::Refines => "Refines".to_string(),
            TraceLinkType::VerifiedBy => "VerifiedBy".to_string(),
            _ => "Traces".to_string(),
        }
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
    async fn test_doors_connection() {
        let config = DOORSConfig {
            server_url: "https://doors.company.com".to_string(),
            database: "DoorsDB".to_string(),
            project: "AFCS".to_string(),
            module: "System Requirements".to_string(),
            auth: RMAuthentication::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
        };
        
        let connector = DOORSConnector::new(config);
        assert_eq!(connector.name(), "DOORS Classic");
    }
}
