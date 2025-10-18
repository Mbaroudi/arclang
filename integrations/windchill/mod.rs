use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::plm_integration::*;

pub struct WindchillConnector {
    client: Client,
    config: WindchillConfig,
    session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindchillConfig {
    pub base_url: String,
    pub context: String,
    pub product: String,
    pub organization: String,
    pub library: String,
    pub auth: AuthenticationMethod,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindchillPart {
    #[serde(rename = "@id")]
    id: String,
    number: String,
    name: String,
    version: String,
    state: String,
    #[serde(rename = "type")]
    part_type: String,
    description: Option<String>,
    source: Option<String>,
    unit_cost: Option<f64>,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindchillBOM {
    #[serde(rename = "@id")]
    id: String,
    parent: String,
    structure_type: String,
    members: Vec<WindchillBOMMember>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindchillBOMMember {
    #[serde(rename = "@id")]
    id: String,
    part_number: String,
    quantity: f64,
    unit: String,
    find_number: Option<String>,
    reference_designator: Option<String>,
}

#[derive(Debug, Serialize)]
struct WindchillChangeRequest {
    name: String,
    description: String,
    reason: String,
    affected_objects: Vec<String>,
    change_type: String,
}

impl WindchillConnector {
    pub fn new(config: WindchillConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
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
    
    async fn authenticate(&mut self) -> Result<(), PLMError> {
        match &self.config.auth {
            AuthenticationMethod::OAuth2 { client_id, client_secret, token_url } => {
                let params = [
                    ("grant_type", "client_credentials"),
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                ];
                
                let response = self.client
                    .post(token_url)
                    .form(&params)
                    .send()
                    .await
                    .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
                
                if !response.status().is_success() {
                    return Err(PLMError::AuthenticationError(
                        format!("OAuth2 failed: {}", response.status())
                    ));
                }
                
                #[derive(Deserialize)]
                struct TokenResponse {
                    access_token: String,
                }
                
                let token: TokenResponse = response.json().await
                    .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
                
                self.session_token = Some(token.access_token);
            }
            
            AuthenticationMethod::BasicAuth { username, password } => {
                let auth_header = format!(
                    "Basic {}",
                    base64::encode(format!("{}:{}", username, password))
                );
                self.session_token = Some(auth_header);
            }
            
            AuthenticationMethod::APIKey { key, .. } => {
                self.session_token = Some(key.clone());
            }
            
            _ => {
                return Err(PLMError::AuthenticationError(
                    "Unsupported authentication method".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn build_url(&self, path: &str) -> String {
        format!("{}{}{}", self.config.base_url, self.config.context, path)
    }
    
    async fn get_with_auth(&self, path: &str) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.get(&url);
        
        if let Some(token) = &self.session_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    async fn post_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.post(&url).json(body);
        
        if let Some(token) = &self.session_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    async fn put_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.put(&url).json(body);
        
        if let Some(token) = &self.session_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    fn convert_to_plm_part(&self, wc_part: WindchillPart) -> PLMPart {
        PLMPart {
            id: wc_part.id,
            part_number: wc_part.number,
            revision: wc_part.version,
            name: wc_part.name,
            description: wc_part.description,
            part_type: wc_part.part_type,
            lifecycle_state: self.map_lifecycle_state(&wc_part.state),
            manufacturer: wc_part.source,
            supplier: None,
            unit_cost: wc_part.unit_cost,
            lead_time_weeks: None,
            weight_kg: None,
            material: None,
            safety_level: None,
            custom_attributes: wc_part.attributes.into_iter()
                .map(|(k, v)| (k, self.convert_json_value(v)))
                .collect(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            created_by: "windchill".to_string(),
            modified_by: "windchill".to_string(),
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
    
    fn map_lifecycle_state(&self, wc_state: &str) -> LifecycleState {
        match wc_state {
            "INWORK" => LifecycleState::InWork,
            "UNDERREVIEW" => LifecycleState::UnderReview,
            "RELEASED" => LifecycleState::Released,
            "OBSOLETE" => LifecycleState::Obsolete,
            _ => LifecycleState::InWork,
        }
    }
    
    fn map_to_windchill_state(&self, state: &LifecycleState) -> &str {
        match state {
            LifecycleState::InWork => "INWORK",
            LifecycleState::UnderReview => "UNDERREVIEW",
            LifecycleState::Released => "RELEASED",
            LifecycleState::Obsolete => "OBSOLETE",
            LifecycleState::Frozen => "RELEASED",
        }
    }
}

#[async_trait]
impl PLMConnector for WindchillConnector {
    fn name(&self) -> &str {
        "Windchill"
    }
    
    async fn connect(&mut self, _config: &PLMConfig) -> Result<(), PLMError> {
        self.authenticate().await?;
        
        let response = self.get_with_auth("/ProdMgmt/products").await?;
        
        if !response.status().is_success() {
            return Err(PLMError::ConnectionError(
                format!("Failed to connect: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), PLMError> {
        self.session_token = None;
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<PLMBaseline, PLMError> {
        let product_path = format!(
            "/ProdMgmt/products/{}/parts",
            self.config.product
        );
        
        let response = self.get_with_auth(&product_path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch baseline: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct PartsResponse {
            parts: Vec<WindchillPart>,
        }
        
        let parts_response: PartsResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        let parts: HashMap<String, PLMPart> = parts_response.parts
            .into_iter()
            .map(|wc_part| {
                let part = self.convert_to_plm_part(wc_part);
                (part.part_number.clone(), part)
            })
            .collect();
        
        Ok(PLMBaseline {
            timestamp: chrono::Utc::now(),
            model_hash: String::new(),
            parts,
            boms: HashMap::new(),
            metadata: BaselineMetadata {
                source_system: "Windchill".to_string(),
                version: "12.0".to_string(),
                created_by: "arclang".to_string(),
                project: self.config.product.clone(),
            },
        })
    }
    
    async fn fetch_part(&self, part_number: &str) -> Result<PLMPart, PLMError> {
        let path = format!("/ProdMgmt/parts/{}", part_number);
        
        let response = self.get_with_auth(&path).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PLMError::PartNotFound(part_number.to_string()));
        }
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch part: {}", response.status())
            ));
        }
        
        let wc_part: WindchillPart = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(self.convert_to_plm_part(wc_part))
    }
    
    async fn fetch_bom(&self, parent_part: &str) -> Result<BOM, PLMError> {
        let path = format!("/ProdMgmt/parts/{}/bom", parent_part);
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch BOM: {}", response.status())
            ));
        }
        
        let wc_bom: WindchillBOM = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        let items = wc_bom.members.into_iter()
            .enumerate()
            .map(|(idx, member)| BOMItem {
                item_number: (idx + 1) as u32,
                part_number: member.part_number,
                quantity: member.quantity,
                unit: member.unit,
                reference_designator: member.reference_designator,
                find_number: member.find_number,
                notes: None,
            })
            .collect();
        
        Ok(BOM {
            parent_part: parent_part.to_string(),
            structure_type: wc_bom.structure_type,
            items,
            effectivity: None,
        })
    }
    
    async fn push_changes(&self, delta: &PLMDelta) -> Result<PLMSyncResult, PLMError> {
        let mut result = PLMSyncResult {
            success: true,
            parts_created: Vec::new(),
            parts_updated: Vec::new(),
            parts_failed: Vec::new(),
            eco_id: None,
            sync_timestamp: chrono::Utc::now(),
        };
        
        for part in &delta.added_parts {
            match self.create_part(part).await {
                Ok(id) => result.parts_created.push(id),
                Err(e) => {
                    result.parts_failed.push((part.part_number.clone(), e.to_string()));
                    result.success = false;
                }
            }
        }
        
        for part_diff in &delta.modified_parts {
            let changes = PartChanges {
                description: None,
                lifecycle_state: None,
                supplier: None,
                unit_cost: None,
                custom_attributes: HashMap::new(),
            };
            
            match self.update_part(&part_diff.part_id, &changes).await {
                Ok(_) => result.parts_updated.push(part_diff.part_number.clone()),
                Err(e) => {
                    result.parts_failed.push((part_diff.part_number.clone(), e.to_string()));
                    result.success = false;
                }
            }
        }
        
        if delta.eco_required {
            let change_request = ChangeRequest {
                title: delta.change_summary.clone(),
                description: "Automated sync from ArcLang".to_string(),
                reason: "Model update".to_string(),
                affected_items: delta.affected_part_numbers(),
                requester: "arclang".to_string(),
                priority: Priority::Medium,
                change_type: ECOChangeType::Engineering,
            };
            
            match self.create_eco(&change_request).await {
                Ok(eco_id) => result.eco_id = Some(eco_id),
                Err(e) => {
                    result.parts_failed.push(("ECO".to_string(), e.to_string()));
                }
            }
        }
        
        Ok(result)
    }
    
    async fn create_part(&self, part: &PLMPart) -> Result<String, PLMError> {
        let wc_part = WindchillPart {
            id: String::new(),
            number: part.part_number.clone(),
            name: part.name.clone(),
            version: part.revision.clone(),
            state: self.map_to_windchill_state(&part.lifecycle_state).to_string(),
            part_type: part.part_type.clone(),
            description: part.description.clone(),
            source: part.manufacturer.clone(),
            unit_cost: part.unit_cost,
            attributes: HashMap::new(),
        };
        
        let response = self.post_with_auth("/ProdMgmt/parts", &wc_part).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to create part: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }
        
        let create_response: CreateResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(create_response.id)
    }
    
    async fn update_part(&self, part_id: &str, changes: &PartChanges) -> Result<(), PLMError> {
        let path = format!("/ProdMgmt/parts/{}", part_id);
        
        let response = self.put_with_auth(&path, changes).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to update part: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn create_eco(&self, request: &ChangeRequest) -> Result<String, PLMError> {
        let wc_change = WindchillChangeRequest {
            name: request.title.clone(),
            description: request.description.clone(),
            reason: request.reason.clone(),
            affected_objects: request.affected_items.clone(),
            change_type: format!("{:?}", request.change_type),
        };
        
        let response = self.post_with_auth("/ChangeMgmt/changes", &wc_change).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to create ECO: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct ECOResponse {
            id: String,
            number: String,
        }
        
        let eco_response: ECOResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(eco_response.number)
    }
    
    async fn query_parts(&self, filter: &PartFilter) -> Result<Vec<PLMPart>, PLMError> {
        let mut query_params = Vec::new();
        
        if let Some(part_type) = &filter.part_type {
            query_params.push(format!("type={}", part_type));
        }
        
        if let Some(state) = &filter.lifecycle_state {
            query_params.push(format!("state={}", self.map_to_windchill_state(state)));
        }
        
        let query_string = query_params.join("&");
        let path = format!("/ProdMgmt/parts?{}", query_string);
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to query parts: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct QueryResponse {
            parts: Vec<WindchillPart>,
        }
        
        let query_response: QueryResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(query_response.parts.into_iter()
            .map(|wc_part| self.convert_to_plm_part(wc_part))
            .collect())
    }
    
    async fn check_out(&self, part_id: &str) -> Result<(), PLMError> {
        let path = format!("/ProdMgmt/parts/{}/checkout", part_id);
        
        let response = self.post_with_auth(&path, &serde_json::json!({})).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to check out part: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn check_in(&self, part_id: &str, comment: &str) -> Result<(), PLMError> {
        let path = format!("/ProdMgmt/parts/{}/checkin", part_id);
        
        let body = serde_json::json!({
            "comment": comment
        });
        
        let response = self.post_with_auth(&path, &body).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to check in part: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn get_lifecycle_state(&self, part_id: &str) -> Result<LifecycleState, PLMError> {
        let part = self.fetch_part(part_id).await?;
        Ok(part.lifecycle_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_windchill_connection() {
        let config = WindchillConfig {
            base_url: "https://test.windchill.com".to_string(),
            context: "/Windchill".to_string(),
            product: "TEST-PRODUCT".to_string(),
            organization: "Test Org".to_string(),
            library: "Engineering".to_string(),
            auth: AuthenticationMethod::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
        };
        
        let connector = WindchillConnector::new(config);
        assert_eq!(connector.name(), "Windchill");
    }
}
