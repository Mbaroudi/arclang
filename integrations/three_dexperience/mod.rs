use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::plm_integration::*;

pub struct ThreeDExperienceConnector {
    client: Client,
    config: ThreeDExperienceConfig,
    access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeDExperienceConfig {
    pub platform_url: String,
    pub tenant: String,
    pub auth: AuthenticationMethod,
    pub collaborative_space: String,
    pub security_context: SecurityContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub role: String,
    pub organization: String,
    pub project: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreeDXProduct {
    id: String,
    #[serde(rename = "type")]
    object_type: String,
    title: String,
    description: Option<String>,
    revision: String,
    state: String,
    #[serde(rename = "physicalid")]
    physical_id: String,
    owner: String,
    modified: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreeDXRequirement {
    id: String,
    title: String,
    #[serde(rename = "Chapter")]
    chapter: String,
    #[serde(rename = "Criticality")]
    criticality: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreeDXLogicalComponent {
    id: String,
    #[serde(rename = "type")]
    component_type: String,
    name: String,
    representation: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreeDXStructure {
    parent: String,
    children: Vec<ThreeDXStructureMember>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreeDXStructureMember {
    id: String,
    #[serde(rename = "relId")]
    rel_id: String,
    quantity: f64,
    #[serde(rename = "referenceDesignator")]
    reference_designator: Option<String>,
}

#[derive(Debug, Serialize)]
struct ThreeDXCreateObject {
    #[serde(rename = "type")]
    object_type: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ThreeDXModifyObject {
    id: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ThreeDXChangeAction {
    title: String,
    description: String,
    #[serde(rename = "Maturity")]
    maturity: String,
    #[serde(rename = "AffectedItems")]
    affected_items: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ThreeDXSearchResult {
    #[serde(rename = "totalItems")]
    total_items: usize,
    items: Vec<ThreeDXProduct>,
}

#[derive(Debug, Deserialize)]
struct ThreeDXCreateResponse {
    id: String,
    #[serde(rename = "physicalid")]
    physical_id: String,
}

impl ThreeDExperienceConnector {
    pub fn new(config: ThreeDExperienceConfig) -> Self {
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
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            config,
            access_token: None,
        }
    }
    
    async fn authenticate(&mut self) -> Result<(), PLMError> {
        let auth_url = format!("{}/3DPassport/login", self.config.platform_url);
        
        let credentials = match &self.config.auth {
            AuthenticationMethod::OAuth2 { client_id, client_secret, token_url } => {
                return self.authenticate_oauth2(token_url, client_id, client_secret).await;
            }
            AuthenticationMethod::BasicAuth { username, password } => {
                serde_json::json!({
                    "username": username,
                    "password": password,
                    "tenant": self.config.tenant
                })
            }
            _ => {
                return Err(PLMError::AuthenticationError(
                    "Unsupported authentication method".to_string()
                ));
            }
        };
        
        let response = self.client
            .post(&auth_url)
            .json(&credentials)
            .send()
            .await
            .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PLMError::AuthenticationError(
                format!("3DPassport login failed: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct LoginResponse {
            #[serde(rename = "accessToken")]
            access_token: String,
        }
        
        let login_response: LoginResponse = response.json().await
            .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
        
        self.access_token = Some(login_response.access_token);
        
        Ok(())
    }
    
    async fn authenticate_oauth2(&mut self, token_url: &str, client_id: &str, client_secret: &str) 
        -> Result<(), PLMError> {
        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("scope", "3DSpace"),
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
        
        self.access_token = Some(token.access_token);
        
        Ok(())
    }
    
    fn build_url(&self, api_path: &str) -> String {
        format!("{}{}", self.config.platform_url, api_path)
    }
    
    async fn get_with_auth(&self, path: &str) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.get(&url);
        
        if let Some(token) = &self.access_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req = req.header("SecurityContext", self.format_security_context());
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    async fn post_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.post(&url).json(body);
        
        if let Some(token) = &self.access_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req = req.header("SecurityContext", self.format_security_context());
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    async fn put_with_auth(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response, PLMError> {
        let url = self.build_url(path);
        
        let mut req = self.client.put(&url).json(body);
        
        if let Some(token) = &self.access_token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        
        req = req.header("SecurityContext", self.format_security_context());
        
        req.send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))
    }
    
    fn format_security_context(&self) -> String {
        format!(
            "Role:{}.Organization:{}.Project:{}",
            self.config.security_context.role,
            self.config.security_context.organization,
            self.config.security_context.project
        )
    }
    
    fn convert_to_plm_part(&self, dx_product: ThreeDXProduct) -> PLMPart {
        let mut custom_attrs = HashMap::new();
        
        for (key, value) in dx_product.attributes {
            custom_attrs.insert(key, self.convert_json_value(value));
        }
        
        PLMPart {
            id: dx_product.id.clone(),
            part_number: dx_product.physical_id,
            revision: dx_product.revision,
            name: dx_product.title,
            description: dx_product.description,
            part_type: dx_product.object_type,
            lifecycle_state: self.map_lifecycle_state(&dx_product.state),
            manufacturer: None,
            supplier: None,
            unit_cost: None,
            lead_time_weeks: None,
            weight_kg: None,
            material: None,
            safety_level: None,
            custom_attributes: custom_attrs,
            created_at: chrono::Utc::now(),
            modified_at: chrono::DateTime::parse_from_rfc3339(&dx_product.modified)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            created_by: dx_product.owner.clone(),
            modified_by: dx_product.owner,
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
    
    fn map_lifecycle_state(&self, dx_state: &str) -> LifecycleState {
        match dx_state.to_uppercase().as_str() {
            "IN WORK" | "INWORK" => LifecycleState::InWork,
            "REVIEW" | "IN REVIEW" => LifecycleState::UnderReview,
            "RELEASED" | "RELEASE" => LifecycleState::Released,
            "OBSOLETE" => LifecycleState::Obsolete,
            "FROZEN" => LifecycleState::Frozen,
            _ => LifecycleState::InWork,
        }
    }
    
    fn map_to_3dx_state(&self, state: &LifecycleState) -> &str {
        match state {
            LifecycleState::InWork => "In Work",
            LifecycleState::UnderReview => "In Review",
            LifecycleState::Released => "Released",
            LifecycleState::Obsolete => "Obsolete",
            LifecycleState::Frozen => "Frozen",
        }
    }
}

#[async_trait]
impl PLMConnector for ThreeDExperienceConnector {
    fn name(&self) -> &str {
        "3DEXPERIENCE"
    }
    
    async fn connect(&mut self, _config: &PLMConfig) -> Result<(), PLMError> {
        self.authenticate().await?;
        
        let response = self.get_with_auth("/3DSpace/resources/v1/modeler/ping").await?;
        
        if !response.status().is_success() {
            return Err(PLMError::ConnectionError(
                format!("Failed to connect: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), PLMError> {
        if self.access_token.is_some() {
            let logout_url = "/3DPassport/logout";
            let _ = self.post_with_auth(logout_url, &serde_json::json!({})).await;
            self.access_token = None;
        }
        
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<PLMBaseline, PLMError> {
        let search_path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/search?tenant={}&$searchStr=*",
            self.config.tenant
        );
        
        let response = self.get_with_auth(&search_path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch baseline: {}", response.status())
            ));
        }
        
        let search_result: ThreeDXSearchResult = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        let parts: HashMap<String, PLMPart> = search_result.items
            .into_iter()
            .map(|dx_product| {
                let part = self.convert_to_plm_part(dx_product);
                (part.part_number.clone(), part)
            })
            .collect();
        
        Ok(PLMBaseline {
            timestamp: chrono::Utc::now(),
            model_hash: String::new(),
            parts,
            boms: HashMap::new(),
            metadata: BaselineMetadata {
                source_system: "3DEXPERIENCE".to_string(),
                version: "R2023x".to_string(),
                created_by: "arclang".to_string(),
                project: self.config.collaborative_space.clone(),
            },
        })
    }
    
    async fn fetch_part(&self, part_number: &str) -> Result<PLMPart, PLMError> {
        let path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/search?$searchStr=PLM_ExternalID:{}",
            part_number
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch part: {}", response.status())
            ));
        }
        
        let search_result: ThreeDXSearchResult = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        if search_result.items.is_empty() {
            return Err(PLMError::PartNotFound(part_number.to_string()));
        }
        
        Ok(self.convert_to_plm_part(search_result.items.into_iter().next().unwrap()))
    }
    
    async fn fetch_bom(&self, parent_part: &str) -> Result<BOM, PLMError> {
        let part = self.fetch_part(parent_part).await?;
        
        let path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/{}/dseng:EngRepInstance",
            part.id
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to fetch BOM: {}", response.status())
            ));
        }
        
        let structure: ThreeDXStructure = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        let items = structure.children.into_iter()
            .enumerate()
            .map(|(idx, member)| BOMItem {
                item_number: (idx + 1) as u32,
                part_number: member.id,
                quantity: member.quantity,
                unit: "EA".to_string(),
                reference_designator: member.reference_designator,
                find_number: Some(member.rel_id),
                notes: None,
            })
            .collect();
        
        Ok(BOM {
            parent_part: parent_part.to_string(),
            structure_type: "Engineering BOM".to_string(),
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
                reason: "Model synchronization".to_string(),
                affected_items: delta.affected_part_numbers(),
                requester: "arclang".to_string(),
                priority: Priority::Medium,
                change_type: ECOChangeType::Engineering,
            };
            
            match self.create_eco(&change_request).await {
                Ok(eco_id) => result.eco_id = Some(eco_id),
                Err(e) => {
                    result.parts_failed.push(("ChangeAction".to_string(), e.to_string()));
                }
            }
        }
        
        Ok(result)
    }
    
    async fn create_part(&self, part: &PLMPart) -> Result<String, PLMError> {
        let mut attributes = HashMap::new();
        attributes.insert("V_Name".to_string(), serde_json::json!(part.name));
        attributes.insert("PLM_ExternalID".to_string(), serde_json::json!(part.part_number));
        attributes.insert("revision".to_string(), serde_json::json!(part.revision));
        
        if let Some(desc) = &part.description {
            attributes.insert("V_description".to_string(), serde_json::json!(desc));
        }
        
        let create_obj = ThreeDXCreateObject {
            object_type: "VPMReference".to_string(),
            attributes,
        };
        
        let response = self.post_with_auth(
            "/3DSpace/resources/v1/modeler/dseng:EngItem",
            &create_obj
        ).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to create part: {}", response.status())
            ));
        }
        
        let create_response: ThreeDXCreateResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(create_response.id)
    }
    
    async fn update_part(&self, part_id: &str, changes: &PartChanges) -> Result<(), PLMError> {
        let mut attributes = HashMap::new();
        
        if let Some(desc) = &changes.description {
            attributes.insert("V_description".to_string(), serde_json::json!(desc));
        }
        
        if let Some(state) = &changes.lifecycle_state {
            attributes.insert("state".to_string(), 
                serde_json::json!(self.map_to_3dx_state(state)));
        }
        
        let modify_obj = ThreeDXModifyObject {
            id: part_id.to_string(),
            attributes,
        };
        
        let path = format!("/3DSpace/resources/v1/modeler/dseng:EngItem/{}", part_id);
        
        let response = self.put_with_auth(&path, &modify_obj).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to update part: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn create_eco(&self, request: &ChangeRequest) -> Result<String, PLMError> {
        let change_action = ThreeDXChangeAction {
            title: request.title.clone(),
            description: request.description.clone(),
            maturity: "In Work".to_string(),
            affected_items: request.affected_items.clone(),
        };
        
        let response = self.post_with_auth(
            "/3DSpace/resources/v1/modeler/dslc:ChangeAction",
            &change_action
        ).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to create change action: {}", response.status())
            ));
        }
        
        let create_response: ThreeDXCreateResponse = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(create_response.id)
    }
    
    async fn query_parts(&self, filter: &PartFilter) -> Result<Vec<PLMPart>, PLMError> {
        let mut search_str = String::from("*");
        
        if let Some(name) = &filter.name_contains {
            search_str = format!("V_Name:*{}*", name);
        }
        
        let path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/search?$searchStr={}",
            urlencoding::encode(&search_str)
        );
        
        let response = self.get_with_auth(&path).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to query parts: {}", response.status())
            ));
        }
        
        let search_result: ThreeDXSearchResult = response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))?;
        
        Ok(search_result.items.into_iter()
            .map(|dx_product| self.convert_to_plm_part(dx_product))
            .collect())
    }
    
    async fn check_out(&self, part_id: &str) -> Result<(), PLMError> {
        let path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/{}/checkout",
            part_id
        );
        
        let response = self.post_with_auth(&path, &serde_json::json!({})).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to check out: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn check_in(&self, part_id: &str, comment: &str) -> Result<(), PLMError> {
        let path = format!(
            "/3DSpace/resources/v1/modeler/dseng:EngItem/{}/checkin",
            part_id
        );
        
        let body = serde_json::json!({
            "comment": comment
        });
        
        let response = self.post_with_auth(&path, &body).await?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("Failed to check in: {}", response.status())
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
    async fn test_3dexperience_connection() {
        let config = ThreeDExperienceConfig {
            platform_url: "https://test.3dexperience.com".to_string(),
            tenant: "TestTenant".to_string(),
            auth: AuthenticationMethod::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
            collaborative_space: "Engineering".to_string(),
            security_context: SecurityContext {
                role: "VPLMProjectLeader".to_string(),
                organization: "MyCompany".to_string(),
                project: "AFCS".to_string(),
            },
        };
        
        let connector = ThreeDExperienceConnector::new(config);
        assert_eq!(connector.name(), "3DEXPERIENCE");
    }
}
