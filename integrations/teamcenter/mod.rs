use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::plm_integration::*;

pub struct TeamcenterConnector {
    client: Client,
    config: TeamcenterConfig,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamcenterConfig {
    pub base_url: String,
    pub protocol: TeamcenterProtocol,
    pub pool_manager: String,
    pub auth: AuthenticationMethod,
    pub item_type: String,
    pub revision_rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamcenterProtocol {
    SOAHTTP,
    SOAWebServices,
    REST,
}

#[derive(Debug, Serialize, Deserialize)]
struct TCSession {
    session_id: String,
    user_id: String,
    discriminator: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TCItem {
    uid: String,
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "itemType")]
    item_type: String,
    #[serde(rename = "objectName")]
    object_name: String,
    #[serde(rename = "objectDesc")]
    object_desc: Option<String>,
    #[serde(rename = "owningUser")]
    owning_user: String,
    properties: HashMap<String, TCProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TCProperty {
    #[serde(rename = "displayValue")]
    display_value: String,
    #[serde(rename = "dbValue")]
    db_value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TCItemRevision {
    uid: String,
    #[serde(rename = "itemRevisionId")]
    item_revision_id: String,
    #[serde(rename = "revisionRule")]
    revision_rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TCBOMLine {
    uid: String,
    #[serde(rename = "itemId")]
    item_id: String,
    quantity: f64,
    #[serde(rename = "findNumber")]
    find_number: Option<String>,
    #[serde(rename = "referenceDesignator")]
    reference_designator: Option<String>,
    #[serde(rename = "occurrenceNote")]
    occurrence_note: Option<String>,
}

#[derive(Debug, Serialize)]
struct TCSOARequest<T> {
    header: TCSOAHeader,
    body: T,
}

#[derive(Debug, Serialize)]
struct TCSOAHeader {
    session_id: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct CreateItemsRequest {
    items: Vec<CreateItemInput>,
}

#[derive(Debug, Serialize)]
struct CreateItemInput {
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "itemType")]
    item_type: String,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "description")]
    description: Option<String>,
    properties: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CreateItemsResponse {
    outputs: Vec<CreateItemOutput>,
}

#[derive(Debug, Deserialize)]
struct CreateItemOutput {
    uid: String,
    #[serde(rename = "itemId")]
    item_id: String,
}

#[derive(Debug, Serialize)]
struct UpdatePropertiesRequest {
    objects: Vec<UpdateObjectInput>,
}

#[derive(Debug, Serialize)]
struct UpdateObjectInput {
    uid: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct CreateBOMRequest {
    parent_uid: String,
    children: Vec<BOMChildInput>,
}

#[derive(Debug, Serialize)]
struct BOMChildInput {
    item_uid: String,
    quantity: f64,
    #[serde(rename = "findNumber")]
    find_number: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateChangeRequest {
    #[serde(rename = "changeType")]
    change_type: String,
    name: String,
    description: String,
    #[serde(rename = "affectedItems")]
    affected_items: Vec<String>,
}

impl TeamcenterConnector {
    pub fn new(config: TeamcenterConfig) -> Self {
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
    
    async fn authenticate(&mut self) -> Result<(), PLMError> {
        let login_url = format!("{}/tc/JsonRestServices/Core-2011-06-Session/login", self.config.base_url);
        
        let (username, password) = match &self.config.auth {
            AuthenticationMethod::BasicAuth { username, password } => (username.clone(), password.clone()),
            _ => return Err(PLMError::AuthenticationError("Only BasicAuth supported".to_string())),
        };
        
        let login_request = serde_json::json!({
            "credentials": {
                "user": username,
                "password": password,
                "discriminator": "LDAP",
                "locale": "en_US"
            }
        });
        
        let response = self.client
            .post(&login_url)
            .json(&login_request)
            .send()
            .await
            .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PLMError::AuthenticationError(
                format!("Login failed: {}", response.status())
            ));
        }
        
        #[derive(Deserialize)]
        struct LoginResponse {
            #[serde(rename = "sessionId")]
            session_id: String,
        }
        
        let login_response: LoginResponse = response.json().await
            .map_err(|e| PLMError::AuthenticationError(e.to_string()))?;
        
        self.session_id = Some(login_response.session_id);
        
        Ok(())
    }
    
    fn build_url(&self, service: &str, operation: &str) -> String {
        match self.config.protocol {
            TeamcenterProtocol::SOAHTTP | TeamcenterProtocol::SOAWebServices => {
                format!("{}/tc/JsonRestServices/{}/{}", self.config.base_url, service, operation)
            }
            TeamcenterProtocol::REST => {
                format!("{}/tc/rest/{}/{}", self.config.base_url, service, operation)
            }
        }
    }
    
    async fn soa_request<T, R>(&self, service: &str, operation: &str, body: T) -> Result<R, PLMError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(service, operation);
        
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| PLMError::AuthenticationError("Not authenticated".to_string()))?;
        
        let request = TCSOARequest {
            header: TCSOAHeader {
                session_id: session_id.clone(),
                version: "2011-06".to_string(),
            },
            body,
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| PLMError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PLMError::APIError(
                format!("SOA request failed: {}", response.status())
            ));
        }
        
        response.json().await
            .map_err(|e| PLMError::SerializationError(e.to_string()))
    }
    
    fn convert_to_plm_part(&self, tc_item: TCItem, revision: TCItemRevision) -> PLMPart {
        let mut custom_attrs = HashMap::new();
        
        for (key, prop) in tc_item.properties {
            custom_attrs.insert(key, self.convert_tc_property(prop));
        }
        
        PLMPart {
            id: tc_item.uid,
            part_number: tc_item.item_id,
            revision: revision.item_revision_id,
            name: tc_item.object_name,
            description: tc_item.object_desc,
            part_type: tc_item.item_type,
            lifecycle_state: LifecycleState::InWork,
            manufacturer: None,
            supplier: None,
            unit_cost: None,
            lead_time_weeks: None,
            weight_kg: None,
            material: None,
            safety_level: None,
            custom_attributes: custom_attrs,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            created_by: tc_item.owning_user,
            modified_by: tc_item.owning_user,
        }
    }
    
    fn convert_tc_property(&self, prop: TCProperty) -> AttributeValue {
        match prop.db_value {
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
            _ => AttributeValue::String(prop.display_value),
        }
    }
    
    async fn get_item_by_id(&self, item_id: &str) -> Result<(TCItem, TCItemRevision), PLMError> {
        #[derive(Serialize)]
        struct GetItemRequest {
            items: Vec<ItemQuery>,
        }
        
        #[derive(Serialize)]
        struct ItemQuery {
            #[serde(rename = "itemId")]
            item_id: String,
        }
        
        #[derive(Deserialize)]
        struct GetItemResponse {
            items: Vec<TCItem>,
            revisions: Vec<TCItemRevision>,
        }
        
        let request_body = GetItemRequest {
            items: vec![ItemQuery {
                item_id: item_id.to_string(),
            }],
        };
        
        let response: GetItemResponse = self.soa_request(
            "Core-2008-06-DataManagement",
            "loadObjects",
            request_body
        ).await?;
        
        if response.items.is_empty() {
            return Err(PLMError::PartNotFound(item_id.to_string()));
        }
        
        let item = response.items.into_iter().next().unwrap();
        let revision = response.revisions.into_iter().next()
            .unwrap_or_else(|| TCItemRevision {
                uid: item.uid.clone(),
                item_revision_id: "A".to_string(),
                revision_rule: self.config.revision_rule.clone(),
            });
        
        Ok((item, revision))
    }
    
    async fn expand_bom(&self, item_uid: &str) -> Result<Vec<TCBOMLine>, PLMError> {
        #[derive(Serialize)]
        struct ExpandBOMRequest {
            parent_uid: String,
            #[serde(rename = "revisionRule")]
            revision_rule: String,
        }
        
        #[derive(Deserialize)]
        struct ExpandBOMResponse {
            #[serde(rename = "bomLines")]
            bom_lines: Vec<TCBOMLine>,
        }
        
        let request_body = ExpandBOMRequest {
            parent_uid: item_uid.to_string(),
            revision_rule: self.config.revision_rule.clone(),
        };
        
        let response: ExpandBOMResponse = self.soa_request(
            "Cad-2007-01-StructureManagement",
            "expandPSOneLevel",
            request_body
        ).await?;
        
        Ok(response.bom_lines)
    }
}

#[async_trait]
impl PLMConnector for TeamcenterConnector {
    fn name(&self) -> &str {
        "Teamcenter"
    }
    
    async fn connect(&mut self, _config: &PLMConfig) -> Result<(), PLMError> {
        self.authenticate().await?;
        
        let ping_url = format!("{}/tc/JsonRestServices/Core-2011-06-Session/ping", self.config.base_url);
        
        let response = self.client
            .get(&ping_url)
            .send()
            .await
            .map_err(|e| PLMError::ConnectionError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PLMError::ConnectionError(
                format!("Connection test failed: {}", response.status())
            ));
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), PLMError> {
        if self.session_id.is_some() {
            let logout_url = format!("{}/tc/JsonRestServices/Core-2011-06-Session/logout", self.config.base_url);
            
            let _ = self.client
                .post(&logout_url)
                .send()
                .await;
            
            self.session_id = None;
        }
        
        Ok(())
    }
    
    async fn fetch_baseline(&self) -> Result<PLMBaseline, PLMError> {
        #[derive(Serialize)]
        struct QueryRequest {
            query: String,
            #[serde(rename = "maxResults")]
            max_results: i32,
        }
        
        #[derive(Deserialize)]
        struct QueryResponse {
            items: Vec<TCItem>,
            revisions: Vec<TCItemRevision>,
        }
        
        let query_body = QueryRequest {
            query: format!("Type='{}'", self.config.item_type),
            max_results: 1000,
        };
        
        let response: QueryResponse = self.soa_request(
            "Query-2014-11-Finder",
            "performSearch",
            query_body
        ).await?;
        
        let mut parts = HashMap::new();
        
        for (item, revision) in response.items.into_iter()
            .zip(response.revisions.into_iter()) {
            let part = self.convert_to_plm_part(item, revision);
            parts.insert(part.part_number.clone(), part);
        }
        
        Ok(PLMBaseline {
            timestamp: chrono::Utc::now(),
            model_hash: String::new(),
            parts,
            boms: HashMap::new(),
            metadata: BaselineMetadata {
                source_system: "Teamcenter".to_string(),
                version: "12.4".to_string(),
                created_by: "arclang".to_string(),
                project: self.config.item_type.clone(),
            },
        })
    }
    
    async fn fetch_part(&self, part_number: &str) -> Result<PLMPart, PLMError> {
        let (item, revision) = self.get_item_by_id(part_number).await?;
        Ok(self.convert_to_plm_part(item, revision))
    }
    
    async fn fetch_bom(&self, parent_part: &str) -> Result<BOM, PLMError> {
        let (item, _) = self.get_item_by_id(parent_part).await?;
        let bom_lines = self.expand_bom(&item.uid).await?;
        
        let items = bom_lines.into_iter()
            .enumerate()
            .map(|(idx, line)| BOMItem {
                item_number: (idx + 1) as u32,
                part_number: line.item_id,
                quantity: line.quantity,
                unit: "EA".to_string(),
                reference_designator: line.reference_designator,
                find_number: line.find_number,
                notes: line.occurrence_note,
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
                    result.parts_failed.push(("ECO".to_string(), e.to_string()));
                }
            }
        }
        
        Ok(result)
    }
    
    async fn create_part(&self, part: &PLMPart) -> Result<String, PLMError> {
        let mut properties = HashMap::new();
        properties.insert("object_name".to_string(), part.name.clone());
        
        if let Some(desc) = &part.description {
            properties.insert("object_desc".to_string(), desc.clone());
        }
        
        let create_input = CreateItemInput {
            item_id: part.part_number.clone(),
            item_type: self.config.item_type.clone(),
            name: part.name.clone(),
            description: part.description.clone(),
            properties,
        };
        
        let request_body = CreateItemsRequest {
            items: vec![create_input],
        };
        
        let response: CreateItemsResponse = self.soa_request(
            "Core-2008-06-DataManagement",
            "createItems",
            request_body
        ).await?;
        
        if let Some(output) = response.outputs.into_iter().next() {
            Ok(output.uid)
        } else {
            Err(PLMError::APIError("Failed to create item".to_string()))
        }
    }
    
    async fn update_part(&self, part_id: &str, changes: &PartChanges) -> Result<(), PLMError> {
        let mut properties = HashMap::new();
        
        if let Some(desc) = &changes.description {
            properties.insert("object_desc".to_string(), desc.clone());
        }
        
        let update_input = UpdateObjectInput {
            uid: part_id.to_string(),
            properties,
        };
        
        let request_body = UpdatePropertiesRequest {
            objects: vec![update_input],
        };
        
        let _: serde_json::Value = self.soa_request(
            "Core-2010-09-DataManagement",
            "setProperties",
            request_body
        ).await?;
        
        Ok(())
    }
    
    async fn create_eco(&self, request: &ChangeRequest) -> Result<String, PLMError> {
        let tc_change = CreateChangeRequest {
            change_type: format!("{:?}", request.change_type),
            name: request.title.clone(),
            description: request.description.clone(),
            affected_items: request.affected_items.clone(),
        };
        
        #[derive(Deserialize)]
        struct CreateChangeResponse {
            change_uid: String,
            change_number: String,
        }
        
        let response: CreateChangeResponse = self.soa_request(
            "ChangeManagement-2017-06-ChangeManagement",
            "createChangeObjects",
            tc_change
        ).await?;
        
        Ok(response.change_number)
    }
    
    async fn query_parts(&self, filter: &PartFilter) -> Result<Vec<PLMPart>, PLMError> {
        let mut query_parts = vec![format!("Type='{}'", self.config.item_type)];
        
        if let Some(name) = &filter.name_contains {
            query_parts.push(format!("Name contains '{}'", name));
        }
        
        let query = query_parts.join(" AND ");
        
        #[derive(Serialize)]
        struct QueryRequest {
            query: String,
        }
        
        #[derive(Deserialize)]
        struct QueryResponse {
            items: Vec<TCItem>,
            revisions: Vec<TCItemRevision>,
        }
        
        let request_body = QueryRequest { query };
        
        let response: QueryResponse = self.soa_request(
            "Query-2014-11-Finder",
            "performSearch",
            request_body
        ).await?;
        
        Ok(response.items.into_iter()
            .zip(response.revisions.into_iter())
            .map(|(item, revision)| self.convert_to_plm_part(item, revision))
            .collect())
    }
    
    async fn check_out(&self, part_id: &str) -> Result<(), PLMError> {
        #[derive(Serialize)]
        struct CheckoutRequest {
            objects: Vec<String>,
        }
        
        let request_body = CheckoutRequest {
            objects: vec![part_id.to_string()],
        };
        
        let _: serde_json::Value = self.soa_request(
            "Core-2006-03-DataManagement",
            "checkout",
            request_body
        ).await?;
        
        Ok(())
    }
    
    async fn check_in(&self, part_id: &str, comment: &str) -> Result<(), PLMError> {
        #[derive(Serialize)]
        struct CheckinRequest {
            objects: Vec<String>,
            comment: String,
        }
        
        let request_body = CheckinRequest {
            objects: vec![part_id.to_string()],
            comment: comment.to_string(),
        };
        
        let _: serde_json::Value = self.soa_request(
            "Core-2006-03-DataManagement",
            "checkin",
            request_body
        ).await?;
        
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
    async fn test_teamcenter_connection() {
        let config = TeamcenterConfig {
            base_url: "https://test.teamcenter.com".to_string(),
            protocol: TeamcenterProtocol::SOAHTTP,
            pool_manager: "Teamcenter Pool Manager".to_string(),
            auth: AuthenticationMethod::BasicAuth {
                username: "test".to_string(),
                password: "test".to_string(),
            },
            item_type: "Item".to_string(),
            revision_rule: "Latest Working".to_string(),
        };
        
        let connector = TeamcenterConnector::new(config);
        assert_eq!(connector.name(), "Teamcenter");
    }
}
