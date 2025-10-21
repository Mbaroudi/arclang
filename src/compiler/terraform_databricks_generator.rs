// ═══════════════════════════════════════════════════════════════════════════
// TERRAFORM DATABRICKS GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate Terraform code using Databricks provider from ArcLang models
// Target: Databricks Terraform Provider v1.86+ (2025)
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, ComponentInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_terraform_databricks(
    model: &SemanticModel,
    config: &TerraformConfig,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    
    // Header and provider configuration
    output.push_str(&generate_header(config));
    output.push_str("\n");
    
    // Provider configuration
    output.push_str(&generate_provider_config(config));
    output.push_str("\n");
    
    // Generate Unity Catalog resources
    output.push_str(&generate_unity_catalog(model, config));
    output.push_str("\n");
    
    // Generate compute resources (clusters, SQL warehouses)
    output.push_str(&generate_compute_resources(model, config));
    output.push_str("\n");
    
    // Generate jobs and workflows
    output.push_str(&generate_jobs_and_workflows(model, config));
    output.push_str("\n");
    
    // Generate storage and networking
    output.push_str(&generate_storage_resources(model, config));
    output.push_str("\n");
    
    // Generate access control and governance
    output.push_str(&generate_governance_resources(model, config));
    output.push_str("\n");
    
    // Generate monitoring and observability
    output.push_str(&generate_monitoring_resources(model, config));
    output.push_str("\n");
    
    // Generate outputs
    output.push_str(&generate_outputs(model));
    
    Ok(output)
}

#[derive(Debug, Clone)]
pub struct TerraformConfig {
    pub workspace_host: String,
    pub account_id: Option<String>,
    pub cloud_provider: CloudProvider,
    pub region: String,
    pub enable_unity_catalog: bool,
    pub enable_monitoring: bool,
}

#[derive(Debug, Clone)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
}

impl Default for TerraformConfig {
    fn default() -> Self {
        TerraformConfig {
            workspace_host: "https://your-workspace.cloud.databricks.com".to_string(),
            account_id: None,
            cloud_provider: CloudProvider::AWS,
            region: "us-east-1".to_string(),
            enable_unity_catalog: true,
            enable_monitoring: true,
        }
    }
}

fn generate_header(config: &TerraformConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# DATABRICKS INFRASTRUCTURE - GENERATED FROM ARCLANG MODEL
# ═══════════════════════════════════════════════════════════════════════════
# Generated: {}
# Cloud Provider: {:?}
# Region: {}
# Unity Catalog: {}
# ═══════════════════════════════════════════════════════════════════════════

terraform {{
  required_version = ">= 1.5.0"
  
  required_providers {{
    databricks = {{
      source  = "databricks/databricks"
      version = "~> 1.86"
    }}
    aws = {{
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }}
  }}
}}
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.cloud_provider,
        config.region,
        config.enable_unity_catalog
    )
}

fn generate_provider_config(config: &TerraformConfig) -> String {
    let mut output = String::new();
    
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# PROVIDER CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

provider "databricks" {
  host = var.workspace_host
"#);
    
    match config.cloud_provider {
        CloudProvider::AWS => {
            output.push_str("  # AWS authentication via AWS CLI or instance profile\n");
        }
        CloudProvider::Azure => {
            output.push_str("  # Azure authentication via Azure CLI or service principal\n");
            output.push_str("  azure_workspace_resource_id = var.azure_workspace_id\n");
        }
        CloudProvider::GCP => {
            output.push_str("  # GCP authentication via service account\n");
            output.push_str("  google_service_account = var.gcp_service_account\n");
        }
    }
    
    output.push_str("}\n\n");
    
    // Add AWS provider if needed
    if matches!(config.cloud_provider, CloudProvider::AWS) {
        output.push_str(&format!(
r#"provider "aws" {{
  region = "{}"
}}

"#, config.region));
    }
    
    output.push_str(&generate_variables());
    output
}

fn generate_variables() -> String {
r#"# ═══════════════════════════════════════════════════════════════════════════
# VARIABLES
# ═══════════════════════════════════════════════════════════════════════════

variable "workspace_host" {
  description = "Databricks workspace URL"
  type        = string
}

variable "catalog_name" {
  description = "Unity Catalog name"
  type        = string
  default     = "data_platform"
}

variable "metastore_name" {
  description = "Unity Catalog metastore name"
  type        = string
  default     = "data_platform_metastore"
}

variable "environment" {
  description = "Environment (dev, staging, prod)"
  type        = string
  default     = "prod"
}

"#.to_string()
}

fn generate_unity_catalog(model: &SemanticModel, config: &TerraformConfig) -> String {
    if !config.enable_unity_catalog {
        return String::new();
    }
    
    let mut output = String::new();
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# UNITY CATALOG - DATA GOVERNANCE
# ═══════════════════════════════════════════════════════════════════════════

# Unity Catalog Metastore
resource "databricks_metastore" "this" {
  name          = var.metastore_name
  storage_root  = "s3://${aws_s3_bucket.unity_catalog.bucket}/metastore"
  owner         = "account users"
  region        = var.region
  force_destroy = false
}

resource "databricks_metastore_assignment" "this" {
  metastore_id = databricks_metastore.this.id
  workspace_id = var.workspace_id
}

# Catalog for data platform
resource "databricks_catalog" "data_platform" {
  name          = var.catalog_name
  comment       = "Data Platform Migration System - Managed by Terraform"
  properties = {
    purpose      = "data_migration"
    owner_team   = "data_engineering"
    managed_by   = "terraform"
  }
  
  depends_on = [databricks_metastore_assignment.this]
}

"#);
    
    // Generate schemas based on logical architecture layers
    let schemas = extract_schemas_from_model(model);
    for schema in schemas {
        output.push_str(&format!(
r#"# Schema: {}
resource "databricks_schema" "{}" {{
  catalog_name = databricks_catalog.data_platform.name
  name         = "{}"
  comment      = "{}"
  properties = {{
    layer = "{}"
  }}
}}

"#,
            schema.display_name,
            schema.resource_name,
            schema.name,
            schema.description,
            schema.layer
        ));
    }
    
    // Generate external location for Delta Lake
    output.push_str(
r#"# External Location for Delta Lake storage
resource "databricks_external_location" "delta_lake" {
  name            = "delta_lake_storage"
  url             = "s3://${aws_s3_bucket.delta_lake.bucket}/data"
  credential_name = databricks_storage_credential.delta_lake.name
  comment         = "Delta Lake storage for data platform"
  
  depends_on = [
    databricks_metastore_assignment.this
  ]
}

"#);
    
    output
}

fn generate_compute_resources(model: &SemanticModel, _config: &TerraformConfig) -> String {
    let mut output = String::new();
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE RESOURCES - CLUSTERS & SQL WAREHOUSES
# ═══════════════════════════════════════════════════════════════════════════

# All-purpose cluster for data engineering
resource "databricks_cluster" "data_engineering" {
  cluster_name            = "data-engineering-cluster"
  spark_version           = data.databricks_spark_version.latest_lts.id
  node_type_id            = data.databricks_node_type.smallest.id
  autotermination_minutes = 20
  
  autoscale {
    min_workers = 2
    max_workers = 200
  }
  
  spark_conf = {
    "spark.databricks.delta.preview.enabled"                = "true"
    "spark.databricks.delta.optimizeWrite.enabled"          = "true"
    "spark.databricks.delta.autoCompact.enabled"            = "true"
  }
  
  custom_tags = {
    component     = "LA-PROC-001"
    requirement   = "SYS-SCALE-001"
    environment   = var.environment
    managed_by    = "terraform"
  }
}

"#);
    
    // Generate streaming cluster
    output.push_str(
r#"# Streaming processing cluster
resource "databricks_cluster" "streaming" {
  cluster_name            = "streaming-pipeline-cluster"
  spark_version           = data.databricks_spark_version.latest_lts.id
  node_type_id            = "i3.xlarge"
  autotermination_minutes = 0  # Always-on for streaming
  
  autoscale {
    min_workers = 2
    max_workers = 50
  }
  
  spark_conf = {
    "spark.streaming.stopGracefullyOnShutdown" = "true"
    "spark.sql.streaming.minBatchesToRetain"   = "10"
  }
  
  custom_tags = {
    component     = "LA-PROC-002"
    requirement   = "SYS-PERF-004"
    safety_level  = "Medium"
    managed_by    = "terraform"
  }
}

"#);
    
    // Generate SQL Analytics warehouse
    let sql_components = model.components.iter()
        .filter(|c| c.name.contains("SQL") || c.name.contains("Analytics"))
        .collect::<Vec<_>>();
    
    if !sql_components.is_empty() {
        output.push_str(
r#"# SQL Analytics Warehouse
resource "databricks_sql_endpoint" "analytics" {
  name             = "analytics-sql-warehouse"
  cluster_size     = "2X-Large"
  max_num_clusters = 3
  auto_stop_mins   = 10
  
  enable_serverless_compute = true
  enable_photon             = true
  
  tags {
    custom_tags = {
      component    = "LA-ANLZ-001"
      requirement  = "SYS-PERF-001"
      safety_level = "Low"
      managed_by   = "terraform"
    }
  }
}

"#);
    }
    
    // Data sources for latest versions
    output.push_str(
r#"# Data sources
data "databricks_spark_version" "latest_lts" {
  long_term_support = true
}

data "databricks_node_type" "smallest" {
  local_disk = true
}

"#);
    
    output
}

fn generate_jobs_and_workflows(model: &SemanticModel, _config: &TerraformConfig) -> String {
    let mut output = String::new();
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# JOBS & WORKFLOWS - ETL ORCHESTRATION
# ═══════════════════════════════════════════════════════════════════════════

# Data migration job
resource "databricks_job" "data_migration" {
  name = "data-migration-etl"
  
  job_cluster {
    job_cluster_key = "migration_cluster"
    new_cluster {
      num_workers   = 8
      spark_version = data.databricks_spark_version.latest_lts.id
      node_type_id  = "c6g.8xlarge"
      
      spark_conf = {
        "spark.databricks.delta.optimizeWrite.enabled" = "true"
      }
    }
  }
  
  task {
    task_key = "extract_oracle"
    
    job_cluster_key = "migration_cluster"
    
    notebook_task {
      notebook_path = "/Shared/migration/extract_oracle"
    }
  }
  
  task {
    task_key = "extract_snowflake"
    
    job_cluster_key = "migration_cluster"
    
    notebook_task {
      notebook_path = "/Shared/migration/extract_snowflake"
    }
  }
  
  task {
    task_key = "schema_conversion"
    depends_on {
      task_key = "extract_oracle"
    }
    depends_on {
      task_key = "extract_snowflake"
    }
    
    job_cluster_key = "migration_cluster"
    
    notebook_task {
      notebook_path = "/Shared/migration/schema_converter"
    }
  }
  
  task {
    task_key = "data_validation"
    depends_on {
      task_key = "schema_conversion"
    }
    
    job_cluster_key = "migration_cluster"
    
    notebook_task {
      notebook_path = "/Shared/migration/data_validator"
    }
  }
  
  schedule {
    quartz_cron_expression = "0 0 2 * * ?"  # Daily at 2 AM
    timezone_id            = "America/New_York"
  }
  
  email_notifications {
    on_failure = ["data-engineering@company.com"]
  }
  
  tags = {
    component    = "LA-MIG-001"
    requirement  = "SYS-MIG-005"
    safety_level = "High"
  }
}

"#);
    
    output
}

fn generate_storage_resources(model: &SemanticModel, config: &TerraformConfig) -> String {
    let mut output = String::new();
    
    if !matches!(config.cloud_provider, CloudProvider::AWS) {
        return output;
    }
    
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# STORAGE RESOURCES - S3, DELTA LAKE
# ═══════════════════════════════════════════════════════════════════════════

# S3 bucket for Unity Catalog metastore
resource "aws_s3_bucket" "unity_catalog" {
  bucket = "databricks-unity-catalog-${var.environment}"
  
  tags = {
    Name        = "Unity Catalog Metastore"
    component   = "LA-TGT-002"
    requirement = "SYS-GOV-003"
    managed_by  = "terraform"
  }
}

resource "aws_s3_bucket_versioning" "unity_catalog" {
  bucket = aws_s3_bucket.unity_catalog.id
  
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "unity_catalog" {
  bucket = aws_s3_bucket.unity_catalog.id
  
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# S3 bucket for Delta Lake storage
resource "aws_s3_bucket" "delta_lake" {
  bucket = "databricks-delta-lake-${var.environment}"
  
  tags = {
    Name        = "Delta Lake Storage"
    component   = "LA-TGT-003"
    requirement = "SYS-PERF-002"
    managed_by  = "terraform"
  }
}

resource "aws_s3_bucket_versioning" "delta_lake" {
  bucket = aws_s3_bucket.delta_lake.id
  
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "delta_lake" {
  bucket = aws_s3_bucket.delta_lake.id
  
  rule {
    id     = "archive_old_data"
    status = "Enabled"
    
    transition {
      days          = 90
      storage_class = "GLACIER_IR"
    }
    
    transition {
      days          = 180
      storage_class = "GLACIER"
    }
  }
}

# IAM role for Databricks
resource "aws_iam_role" "databricks_storage" {
  name = "databricks-storage-access-${var.environment}"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::414351767826:role/unity-catalog-prod-UCMasterRole"
        }
        Action = "sts:AssumeRole"
        Condition = {
          StringEquals = {
            "sts:ExternalId" = var.databricks_account_id
          }
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "databricks_storage" {
  name = "databricks-storage-policy"
  role = aws_iam_role.databricks_storage.id
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket",
          "s3:GetBucketLocation"
        ]
        Resource = [
          "${aws_s3_bucket.delta_lake.arn}/*",
          aws_s3_bucket.delta_lake.arn
        ]
      }
    ]
  })
}

# Storage credential for Unity Catalog
resource "databricks_storage_credential" "delta_lake" {
  name = "delta_lake_credential"
  
  aws_iam_role {
    role_arn = aws_iam_role.databricks_storage.arn
  }
  
  comment = "Storage credential for Delta Lake on S3"
  
  depends_on = [
    databricks_metastore_assignment.this
  ]
}

"#);
    
    output
}

fn generate_governance_resources(model: &SemanticModel, _config: &TerraformConfig) -> String {
    let mut output = String::new();
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# GOVERNANCE & ACCESS CONTROL
# ═══════════════════════════════════════════════════════════════════════════

# Groups
resource "databricks_group" "data_engineers" {
  display_name = "data-engineers"
}

resource "databricks_group" "data_analysts" {
  display_name = "data-analysts"
}

resource "databricks_group" "data_scientists" {
  display_name = "data-scientists"
}

# Catalog permissions
resource "databricks_grants" "catalog" {
  catalog = databricks_catalog.data_platform.name
  
  grant {
    principal  = databricks_group.data_engineers.display_name
    privileges = ["USE_CATALOG", "USE_SCHEMA", "CREATE_SCHEMA", "CREATE_TABLE"]
  }
  
  grant {
    principal  = databricks_group.data_analysts.display_name
    privileges = ["USE_CATALOG", "USE_SCHEMA", "SELECT"]
  }
  
  grant {
    principal  = databricks_group.data_scientists.display_name
    privileges = ["USE_CATALOG", "USE_SCHEMA", "SELECT", "CREATE_TABLE"]
  }
}

# Cluster policies
resource "databricks_cluster_policy" "data_engineering" {
  name = "data-engineering-policy"
  
  definition = jsonencode({
    "spark_version" : {
      "type" : "fixed",
      "value" : "auto:latest-lts"
    },
    "node_type_id" : {
      "type" : "allowlist",
      "values" : ["i3.xlarge", "i3.2xlarge", "i3.4xlarge"]
    },
    "autotermination_minutes" : {
      "type" : "range",
      "minValue" : 10,
      "maxValue" : 120
    }
  })
}

"#);
    
    // Add governance tags based on requirements
    let governance_components = model.components.iter()
        .filter(|c| c.name.contains("Access Control") || 
                    c.name.contains("Governance") || 
                    c.name.contains("Audit"))
        .collect::<Vec<_>>();
    
    if governance_components.len() > 0 {
        output.push_str(
r#"# Audit logging
resource "databricks_system_schema" "access_logs" {
  schema_name = "access"
}

resource "databricks_system_schema" "audit_logs" {
  schema_name = "audit"
}

"#);
    }
    
    output
}

fn generate_monitoring_resources(model: &SemanticModel, config: &TerraformConfig) -> String {
    if !config.enable_monitoring {
        return String::new();
    }
    
    let mut output = String::new();
    output.push_str(
r#"# ═══════════════════════════════════════════════════════════════════════════
# MONITORING & OBSERVABILITY
# ═══════════════════════════════════════════════════════════════════════════

# Alert destinations
resource "databricks_notification_destination" "email" {
  display_name = "Data Engineering Team"
  
  config {
    email {
      addresses = ["data-engineering@company.com"]
    }
  }
}

resource "databricks_notification_destination" "slack" {
  display_name = "Slack Alerts"
  
  config {
    slack {
      url = var.slack_webhook_url
    }
  }
}

# Cluster usage monitoring
resource "databricks_sql_query" "cluster_usage" {
  data_source_id = databricks_sql_endpoint.analytics.data_source_id
  name           = "Cluster Usage Monitoring"
  
  query = <<-EOT
    SELECT
      cluster_id,
      cluster_name,
      SUM(usage_quantity) as total_dbu,
      COUNT(DISTINCT usage_date) as days_active
    FROM system.billing.usage
    WHERE usage_date >= CURRENT_DATE - INTERVAL 30 DAYS
    GROUP BY cluster_id, cluster_name
    ORDER BY total_dbu DESC
  EOT
}

# Cost tracking dashboard
resource "databricks_dashboard" "cost_tracking" {
  display_name = "Cost Tracking Dashboard"
  warehouse_id = databricks_sql_endpoint.analytics.id
  
  tags = {
    component   = "LA-MON-002"
    requirement = "SYS-MON-003"
  }
}

"#);
    
    output
}

fn generate_outputs(model: &SemanticModel) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# OUTPUTS
# ═══════════════════════════════════════════════════════════════════════════

output "catalog_name" {{
  description = "Unity Catalog name"
  value       = databricks_catalog.data_platform.name
}}

output "metastore_id" {{
  description = "Unity Catalog metastore ID"
  value       = databricks_metastore.this.id
}}

output "data_engineering_cluster_id" {{
  description = "Data engineering cluster ID"
  value       = databricks_cluster.data_engineering.id
}}

output "sql_warehouse_id" {{
  description = "SQL Analytics warehouse ID"
  value       = databricks_sql_endpoint.analytics.id
}}

output "delta_lake_bucket" {{
  description = "S3 bucket for Delta Lake storage"
  value       = aws_s3_bucket.delta_lake.bucket
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

// Helper struct for schema generation
struct SchemaInfo {
    name: String,
    resource_name: String,
    display_name: String,
    description: String,
    layer: String,
}

fn extract_schemas_from_model(model: &SemanticModel) -> Vec<SchemaInfo> {
    let mut schemas = Vec::new();
    
    // Extract unique layers from components
    let mut layers = std::collections::HashSet::new();
    for component in &model.components {
        if let Some(layer) = extract_layer_from_component(component) {
            layers.insert(layer);
        }
    }
    
    // Create schema for each layer
    for layer in layers {
        let schema_name = layer.to_lowercase().replace(" ", "_");
        schemas.push(SchemaInfo {
            name: schema_name.clone(),
            resource_name: schema_name.clone(),
            display_name: layer.clone(),
            description: format!("{} layer data", layer),
            layer: layer,
        });
    }
    
    // Default schemas
    if schemas.is_empty() {
        schemas.push(SchemaInfo {
            name: "bronze".to_string(),
            resource_name: "bronze".to_string(),
            display_name: "Bronze".to_string(),
            description: "Raw ingested data".to_string(),
            layer: "ingestion".to_string(),
        });
        schemas.push(SchemaInfo {
            name: "silver".to_string(),
            resource_name: "silver".to_string(),
            display_name: "Silver".to_string(),
            description: "Cleansed and conformed data".to_string(),
            layer: "processing".to_string(),
        });
        schemas.push(SchemaInfo {
            name: "gold".to_string(),
            resource_name: "gold".to_string(),
            display_name: "Gold".to_string(),
            description: "Business-level aggregates".to_string(),
            layer: "analytics".to_string(),
        });
    }
    
    schemas
}

fn extract_layer_from_component(component: &ComponentInfo) -> Option<String> {
    let name = component.name.to_lowercase();
    
    if name.contains("source") || name.contains("oracle") || name.contains("snowflake") {
        Some("Source".to_string())
    } else if name.contains("migration") || name.contains("etl") {
        Some("Migration".to_string())
    } else if name.contains("processing") || name.contains("pipeline") {
        Some("Processing".to_string())
    } else if name.contains("governance") || name.contains("catalog") {
        Some("Governance".to_string())
    } else if name.contains("analytics") || name.contains("sql") {
        Some("Analytics".to_string())
    } else if name.contains("monitoring") {
        Some("Monitoring".to_string())
    } else {
        None
    }
}
