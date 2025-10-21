// ═══════════════════════════════════════════════════════════════════════════
// AZURE COMPLETE TERRAFORM GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate complete Azure infrastructure from ArcLang physical architecture
// Coverage: AKS, Synapse, Azure Data Lake, Event Hub, Functions, Monitor
// Target: AzureRM Provider v4.0+ (2025)
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, ComponentInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_azure_terraform(
    model: &SemanticModel,
    config: &AzureConfig,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    
    output.push_str(&generate_azure_header(config));
    output.push_str("\n");
    output.push_str(&generate_azure_providers(config));
    output.push_str("\n");
    output.push_str(&generate_azure_variables());
    output.push_str("\n");
    
    // Core infrastructure
    output.push_str(&generate_azure_networking(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_data_layer(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_compute_aks(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_synapse(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_functions(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_integration(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_monitoring(model, config));
    output.push_str("\n");
    output.push_str(&generate_azure_outputs(model));
    
    Ok(output)
}

#[derive(Debug, Clone)]
pub struct AzureConfig {
    pub location: String,
    pub resource_group_name: String,
    pub environment: String,
    pub project_name: String,
}

impl Default for AzureConfig {
    fn default() -> Self {
        AzureConfig {
            location: "East US".to_string(),
            resource_group_name: "data-platform-rg".to_string(),
            environment: "prod".to_string(),
            project_name: "data-platform".to_string(),
        }
    }
}

fn generate_azure_header(config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# AZURE INFRASTRUCTURE - GENERATED FROM ARCLANG MODEL
# ═══════════════════════════════════════════════════════════════════════════
# Generated: {}
# Location: {}
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

terraform {{
  required_version = ">= 1.5.0"
  
  required_providers {{
    azurerm = {{
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }}
    azuread = {{
      source  = "hashicorp/azuread"
      version = "~> 2.0"
    }}
    databricks = {{
      source  = "databricks/databricks"
      version = "~> 1.86"
    }}
  }}
  
  backend "azurerm" {{
    resource_group_name  = "terraform-state-rg"
    storage_account_name = "tfstate"
    container_name       = "tfstate"
    key                  = "{}.terraform.tfstate"
  }}
}}
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.location,
        config.environment,
        config.environment
    )
}

fn generate_azure_providers(config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# PROVIDERS
# ═══════════════════════════════════════════════════════════════════════════

provider "azurerm" {{
  features {{
    resource_group {{
      prevent_deletion_if_contains_resources = true
    }}
    key_vault {{
      purge_soft_delete_on_destroy = false
    }}
  }}
}}

provider "azuread" {{}}

provider "databricks" {{
  host = azurerm_databricks_workspace.main.workspace_url
}}
"#
    )
}

fn generate_azure_variables() -> String {
r#"# ═══════════════════════════════════════════════════════════════════════════
# VARIABLES
# ═══════════════════════════════════════════════════════════════════════════

variable "location" {
  description = "Azure region"
  type        = string
  default     = "East US"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "project_name" {
  description = "Project name"
  type        = string
  default     = "data-platform"
}

"#.to_string()
}

fn generate_azure_networking(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# NETWORKING - VNET, SUBNETS, NSG
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_resource_group" "main" {{
  name     = "${{var.project_name}}-${{var.environment}}-rg"
  location = var.location
  
  tags = {{
    Environment  = var.environment
    ManagedBy    = "Terraform"
    GeneratedFrom = "ArcLang"
  }}
}}

resource "azurerm_virtual_network" "main" {{
  name                = "${{var.project_name}}-vnet"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  address_space       = ["10.0.0.0/16"]
  
  tags = {{
    PhysicalNode = "PA-CLOUD-001"
  }}
}}

resource "azurerm_subnet" "aks" {{
  name                 = "aks-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.1.0/24"]
}}

resource "azurerm_subnet" "data" {{
  name                 = "data-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.2.0/24"]
  
  service_endpoints = ["Microsoft.Storage", "Microsoft.Sql"]
}}

resource "azurerm_subnet" "synapse" {{
  name                 = "synapse-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.3.0/24"]
}}

resource "azurerm_network_security_group" "main" {{
  name                = "${{var.project_name}}-nsg"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  
  security_rule {{
    name                       = "AllowHTTPS"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "443"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }}
}}

"#
    )
}

fn generate_azure_data_layer(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# DATA LAYER - STORAGE ACCOUNT, DATA LAKE, COSMOS DB
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_storage_account" "datalake" {{
  name                     = "${{var.project_name}}datalake"
  resource_group_name      = azurerm_resource_group.main.name
  location                 = azurerm_resource_group.main.location
  account_tier             = "Standard"
  account_replication_type = "GRS"
  account_kind             = "StorageV2"
  is_hns_enabled           = true
  
  blob_properties {{
    versioning_enabled = true
    
    delete_retention_policy {{
      days = 30
    }}
  }}
  
  network_rules {{
    default_action             = "Deny"
    virtual_network_subnet_ids = [azurerm_subnet.data.id]
  }}
  
  tags = {{
    Component = "LA-TGT-003"
    PhysicalNode = "PA-CLOUD-001"
    Layer = "data"
  }}
}}

resource "azurerm_storage_data_lake_gen2_filesystem" "bronze" {{
  name               = "bronze"
  storage_account_id = azurerm_storage_account.datalake.id
  
  properties = {{
    layer = "bronze"
  }}
}}

resource "azurerm_storage_data_lake_gen2_filesystem" "silver" {{
  name               = "silver"
  storage_account_id = azurerm_storage_account.datalake.id
  
  properties = {{
    layer = "silver"
  }}
}}

resource "azurerm_storage_data_lake_gen2_filesystem" "gold" {{
  name               = "gold"
  storage_account_id = azurerm_storage_account.datalake.id
  
  properties = {{
    layer = "gold"
  }}
}}

resource "azurerm_cosmosdb_account" "main" {{
  name                = "${{var.project_name}}-cosmos"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  offer_type          = "Standard"
  kind                = "GlobalDocumentDB"
  
  consistency_policy {{
    consistency_level = "Session"
  }}
  
  geo_location {{
    location          = azurerm_resource_group.main.location
    failover_priority = 0
  }}
  
  capabilities {{
    name = "EnableServerless"
  }}
  
  tags = {{
    Component = "LA-MIG-003"
    Requirement = "SYS-MIG-003"
  }}
}}

resource "azurerm_cosmosdb_sql_database" "migration" {{
  name                = "migration"
  resource_group_name = azurerm_resource_group.main.name
  account_name        = azurerm_cosmosdb_account.main.name
}}

resource "azurerm_cosmosdb_sql_container" "migration_state" {{
  name                = "migration_state"
  resource_group_name = azurerm_resource_group.main.name
  account_name        = azurerm_cosmosdb_account.main.name
  database_name       = azurerm_cosmosdb_sql_database.migration.name
  partition_key_paths = ["/table_name"]
}}

"#
    )
}

fn generate_azure_compute_aks(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE - AZURE KUBERNETES SERVICE (AKS)
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_kubernetes_cluster" "main" {{
  name                = "${{var.project_name}}-aks"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  dns_prefix          = "${{var.project_name}}"
  
  default_node_pool {{
    name                = "system"
    node_count          = 3
    vm_size             = "Standard_D4s_v3"
    vnet_subnet_id      = azurerm_subnet.aks.id
    enable_auto_scaling = true
    min_count           = 3
    max_count           = 10
    
    upgrade_settings {{
      max_surge = "10%"
    }}
  }}
  
  identity {{
    type = "SystemAssigned"
  }}
  
  network_profile {{
    network_plugin    = "azure"
    network_policy    = "azure"
    load_balancer_sku = "standard"
  }}
  
  monitor_metrics {{
    annotations_allowed = null
    labels_allowed      = null
  }}
  
  oms_agent {{
    log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  }}
  
  azure_policy_enabled = true
  
  tags = {{
    Component = "LA-PROC-001"
    PhysicalNode = "PA-DBX-002"
    Requirement = "SYS-SCALE-001"
  }}
}}

resource "azurerm_kubernetes_cluster_node_pool" "data_processing" {{
  name                  = "dataproc"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.main.id
  vm_size               = "Standard_E8s_v3"
  enable_auto_scaling   = true
  min_count             = 2
  max_count             = 50
  vnet_subnet_id        = azurerm_subnet.aks.id
  
  node_labels = {{
    "workload" = "data-processing"
    "component" = "LA-PROC-001"
  }}
  
  node_taints = [
    "workload=data-processing:NoSchedule"
  ]
  
  tags = {{
    Component = "LA-PROC-001"
    Requirement = "SYS-PERF-002"
  }}
}}

"#
    )
}

fn generate_azure_synapse(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# ANALYTICS - AZURE SYNAPSE ANALYTICS
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_synapse_workspace" "main" {{
  name                                 = "${{var.project_name}}-synapse"
  resource_group_name                  = azurerm_resource_group.main.name
  location                             = azurerm_resource_group.main.location
  storage_data_lake_gen2_filesystem_id = azurerm_storage_data_lake_gen2_filesystem.gold.id
  sql_administrator_login              = "sqladminuser"
  sql_administrator_login_password     = random_password.synapse_admin.result
  managed_virtual_network_enabled      = true
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-ANLZ-001"
    PhysicalNode = "PA-ANLZ-001"
    Requirement = "SYS-PERF-001"
  }}
}}

resource "random_password" "synapse_admin" {{
  length  = 32
  special = true
}}

resource "azurerm_synapse_spark_pool" "main" {{
  name                 = "dataprocessing"
  synapse_workspace_id = azurerm_synapse_workspace.main.id
  node_size_family     = "MemoryOptimized"
  node_size            = "Medium"
  cache_size           = 100
  
  auto_scale {{
    max_node_count = 50
    min_node_count = 3
  }}
  
  auto_pause {{
    delay_in_minutes = 15
  }}
  
  spark_version = "3.4"
  
  tags = {{
    Component = "LA-PROC-001"
    Requirement = "SYS-SCALE-001"
  }}
}}

resource "azurerm_synapse_sql_pool" "main" {{
  name                 = "sqldw"
  synapse_workspace_id = azurerm_synapse_workspace.main.id
  sku_name             = "DW100c"
  create_mode          = "Default"
  
  tags = {{
    Component = "LA-ANLZ-001"
  }}
}}

resource "azurerm_synapse_integration_runtime_azure" "main" {{
  name                 = "integration-runtime"
  synapse_workspace_id = azurerm_synapse_workspace.main.id
  location             = azurerm_resource_group.main.location
}}

"#
    )
}

fn generate_azure_functions(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE - AZURE FUNCTIONS (SERVERLESS)
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_service_plan" "functions" {{
  name                = "${{var.project_name}}-functions-plan"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  os_type             = "Linux"
  sku_name            = "EP1"
}}

resource "azurerm_linux_function_app" "schema_converter" {{
  name                = "${{var.project_name}}-schema-converter"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  service_plan_id     = azurerm_service_plan.functions.id
  
  storage_account_name       = azurerm_storage_account.datalake.name
  storage_account_access_key = azurerm_storage_account.datalake.primary_access_key
  
  site_config {{
    application_stack {{
      python_version = "3.11"
    }}
    
    application_insights_connection_string = azurerm_application_insights.main.connection_string
    
    cors {{
      allowed_origins = ["*"]
    }}
  }}
  
  app_settings = {{
    "ENVIRONMENT"    = var.environment
    "COSMOS_ENDPOINT" = azurerm_cosmosdb_account.main.endpoint
  }}
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-MIG-002"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-002"
  }}
}}

resource "azurerm_linux_function_app" "data_validator" {{
  name                = "${{var.project_name}}-data-validator"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  service_plan_id     = azurerm_service_plan.functions.id
  
  storage_account_name       = azurerm_storage_account.datalake.name
  storage_account_access_key = azurerm_storage_account.datalake.primary_access_key
  
  site_config {{
    application_stack {{
      python_version = "3.11"
    }}
    
    application_insights_connection_string = azurerm_application_insights.main.connection_string
  }}
  
  app_settings = {{
    "ENVIRONMENT" = var.environment
  }}
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-MIG-003"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-003"
    SafetyLevel = "Critical"
  }}
}}

"#
    )
}

fn generate_azure_integration(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# INTEGRATION - EVENT HUB, SERVICE BUS, API MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_eventhub_namespace" "main" {{
  name                = "${{var.project_name}}-eventhub"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "Standard"
  capacity            = 2
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-INT-003"
    PhysicalNode = "PA-INT-001"
  }}
}}

resource "azurerm_eventhub" "data_ingestion" {{
  name                = "data-ingestion"
  namespace_name      = azurerm_eventhub_namespace.main.name
  resource_group_name = azurerm_resource_group.main.name
  partition_count     = 10
  message_retention   = 7
  
  capture_description {{
    enabled  = true
    encoding = "Avro"
    
    destination {{
      name                = "EventHubArchive.AzureBlockBlob"
      archive_name_format = "{{Namespace}}/{{EventHub}}/{{PartitionId}}/{{Year}}/{{Month}}/{{Day}}/{{Hour}}/{{Minute}}/{{Second}}"
      blob_container_name = azurerm_storage_data_lake_gen2_filesystem.bronze.name
      storage_account_id  = azurerm_storage_account.datalake.id
    }}
  }}
}}

resource "azurerm_servicebus_namespace" "main" {{
  name                = "${{var.project_name}}-servicebus"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "Premium"
  capacity            = 1
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-INT-003"
  }}
}}

resource "azurerm_servicebus_queue" "etl_jobs" {{
  name         = "etl-jobs"
  namespace_id = azurerm_servicebus_namespace.main.id
  
  enable_partitioning = true
  max_size_in_megabytes = 5120
  
  dead_lettering_on_message_expiration = true
}}

resource "azurerm_api_management" "main" {{
  name                = "${{var.project_name}}-apim"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  publisher_name      = "Data Platform"
  publisher_email     = "admin@example.com"
  sku_name            = "Developer_1"
  
  identity {{
    type = "SystemAssigned"
  }}
  
  tags = {{
    Component = "LA-INT-001"
    PhysicalNode = "PA-INT-001"
  }}
}}

"#
    )
}

fn generate_azure_monitoring(_model: &SemanticModel, config: &AzureConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# MONITORING - LOG ANALYTICS, APPLICATION INSIGHTS
# ═══════════════════════════════════════════════════════════════════════════

resource "azurerm_log_analytics_workspace" "main" {{
  name                = "${{var.project_name}}-logs"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "PerGB2018"
  retention_in_days   = 30
  
  tags = {{
    Component = "LA-MON-001"
    PhysicalNode = "PA-MON-001"
  }}
}}

resource "azurerm_application_insights" "main" {{
  name                = "${{var.project_name}}-appinsights"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  workspace_id        = azurerm_log_analytics_workspace.main.id
  application_type    = "web"
  
  tags = {{
    Component = "LA-MON-001"
    Requirement = "SYS-MON-001"
  }}
}}

resource "azurerm_monitor_action_group" "alerts" {{
  name                = "${{var.project_name}}-alerts"
  resource_group_name = azurerm_resource_group.main.name
  short_name          = "alerts"
  
  email_receiver {{
    name          = "data-engineering"
    email_address = "data-engineering@company.com"
  }}
  
  tags = {{
    Component = "LA-MON-003"
  }}
}}

resource "azurerm_monitor_metric_alert" "aks_cpu" {{
  name                = "aks-high-cpu"
  resource_group_name = azurerm_resource_group.main.name
  scopes              = [azurerm_kubernetes_cluster.main.id]
  description         = "Alert when AKS CPU usage is high"
  
  criteria {{
    metric_namespace = "Microsoft.ContainerService/managedClusters"
    metric_name      = "node_cpu_usage_percentage"
    aggregation      = "Average"
    operator         = "GreaterThan"
    threshold        = 80
  }}
  
  action {{
    action_group_id = azurerm_monitor_action_group.alerts.id
  }}
}}

"#
    )
}

fn generate_azure_outputs(model: &SemanticModel) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# OUTPUTS
# ═══════════════════════════════════════════════════════════════════════════

output "resource_group_name" {{
  value = azurerm_resource_group.main.name
}}

output "aks_cluster_name" {{
  value = azurerm_kubernetes_cluster.main.name
}}

output "aks_kubeconfig" {{
  value     = azurerm_kubernetes_cluster.main.kube_config_raw
  sensitive = true
}}

output "synapse_workspace_url" {{
  value = azurerm_synapse_workspace.main.connectivity_endpoints.web
}}

output "storage_account_name" {{
  value = azurerm_storage_account.datalake.name
}}

output "application_insights_key" {{
  value     = azurerm_application_insights.main.instrumentation_key
  sensitive = true
}}

# Model Statistics
# Requirements: {}
# Components: {}
# Traces: {}
"#,
        model.requirements.len(),
        model.components.len(),
        model.traces.len()
    )
}
