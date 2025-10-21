// ═══════════════════════════════════════════════════════════════════════════
// GCP COMPLETE TERRAFORM GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate complete GCP infrastructure from ArcLang physical architecture
// Coverage: GKE, BigQuery, Cloud Storage, Cloud Functions, Pub/Sub, Monitoring
// Target: Google Provider v6.0+ (2025)
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, ComponentInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_gcp_terraform(
    model: &SemanticModel,
    config: &GcpConfig,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    
    output.push_str(&generate_gcp_header(config));
    output.push_str("\n");
    output.push_str(&generate_gcp_providers(config));
    output.push_str("\n");
    output.push_str(&generate_gcp_variables());
    output.push_str("\n");
    
    output.push_str(&generate_gcp_networking(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_data_layer(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_compute_gke(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_bigquery(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_functions(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_integration(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_orchestration(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_monitoring(model, config));
    output.push_str("\n");
    output.push_str(&generate_gcp_outputs(model));
    
    Ok(output)
}

#[derive(Debug, Clone)]
pub struct GcpConfig {
    pub project_id: String,
    pub region: String,
    pub zone: String,
    pub environment: String,
    pub project_name: String,
}

impl Default for GcpConfig {
    fn default() -> Self {
        GcpConfig {
            project_id: "my-project-id".to_string(),
            region: "us-central1".to_string(),
            zone: "us-central1-a".to_string(),
            environment: "prod".to_string(),
            project_name: "data-platform".to_string(),
        }
    }
}

fn generate_gcp_header(config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# GCP INFRASTRUCTURE - GENERATED FROM ARCLANG MODEL
# ═══════════════════════════════════════════════════════════════════════════
# Generated: {}
# Project: {}
# Region: {}
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

terraform {{
  required_version = ">= 1.5.0"
  
  required_providers {{
    google = {{
      source  = "hashicorp/google"
      version = "~> 6.0"
    }}
    google-beta = {{
      source  = "hashicorp/google-beta"
      version = "~> 6.0"
    }}
  }}
  
  backend "gcs" {{
    bucket = "terraform-state-{}"
    prefix = "terraform/state"
  }}
}}
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.project_id,
        config.region,
        config.environment,
        config.project_id
    )
}

fn generate_gcp_providers(config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# PROVIDERS
# ═══════════════════════════════════════════════════════════════════════════

provider "google" {{
  project = var.project_id
  region  = var.region
}}

provider "google-beta" {{
  project = var.project_id
  region  = var.region
}}
"#
    )
}

fn generate_gcp_variables() -> String {
r#"# ═══════════════════════════════════════════════════════════════════════════
# VARIABLES
# ═══════════════════════════════════════════════════════════════════════════

variable "project_id" {
  description = "GCP project ID"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "zone" {
  description = "GCP zone"
  type        = string
  default     = "us-central1-a"
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

fn generate_gcp_networking(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# NETWORKING - VPC, SUBNETS, FIREWALL
# ═══════════════════════════════════════════════════════════════════════════

resource "google_compute_network" "main" {{
  name                    = "${{var.project_name}}-vpc"
  auto_create_subnetworks = false
  
  project = var.project_id
}}

resource "google_compute_subnetwork" "gke" {{
  name          = "gke-subnet"
  ip_cidr_range = "10.0.0.0/20"
  region        = var.region
  network       = google_compute_network.main.id
  
  secondary_ip_range {{
    range_name    = "gke-pods"
    ip_cidr_range = "10.4.0.0/14"
  }}
  
  secondary_ip_range {{
    range_name    = "gke-services"
    ip_cidr_range = "10.8.0.0/20"
  }}
  
  private_ip_google_access = true
}}

resource "google_compute_subnetwork" "data" {{
  name          = "data-subnet"
  ip_cidr_range = "10.1.0.0/20"
  region        = var.region
  network       = google_compute_network.main.id
  
  private_ip_google_access = true
}}

resource "google_compute_firewall" "allow_internal" {{
  name    = "${{var.project_name}}-allow-internal"
  network = google_compute_network.main.name
  
  allow {{
    protocol = "tcp"
    ports    = ["0-65535"]
  }}
  
  allow {{
    protocol = "udp"
    ports    = ["0-65535"]
  }}
  
  allow {{
    protocol = "icmp"
  }}
  
  source_ranges = ["10.0.0.0/8"]
}}

resource "google_compute_firewall" "allow_https" {{
  name    = "${{var.project_name}}-allow-https"
  network = google_compute_network.main.name
  
  allow {{
    protocol = "tcp"
    ports    = ["443"]
  }}
  
  source_ranges = ["0.0.0.0/0"]
}}

resource "google_compute_router" "main" {{
  name    = "${{var.project_name}}-router"
  region  = var.region
  network = google_compute_network.main.id
}}

resource "google_compute_router_nat" "main" {{
  name                               = "${{var.project_name}}-nat"
  router                             = google_compute_router.main.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}}

"#
    )
}

fn generate_gcp_data_layer(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# DATA LAYER - CLOUD STORAGE, FIRESTORE
# ═══════════════════════════════════════════════════════════════════════════

resource "google_storage_bucket" "bronze" {{
  name          = "${{var.project_name}}-bronze-${{var.environment}}"
  location      = var.region
  force_destroy = false
  
  uniform_bucket_level_access = true
  
  versioning {{
    enabled = true
  }}
  
  lifecycle_rule {{
    condition {{
      age = 90
    }}
    action {{
      type          = "SetStorageClass"
      storage_class = "NEARLINE"
    }}
  }}
  
  lifecycle_rule {{
    condition {{
      age = 180
    }}
    action {{
      type          = "SetStorageClass"
      storage_class = "COLDLINE"
    }}
  }}
  
  encryption {{
    default_kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component     = "la-tgt-003"
    physical_node = "pa-cloud-001"
    layer         = "bronze"
    environment   = var.environment
  }}
}}

resource "google_storage_bucket" "silver" {{
  name          = "${{var.project_name}}-silver-${{var.environment}}"
  location      = var.region
  force_destroy = false
  
  uniform_bucket_level_access = true
  
  versioning {{
    enabled = true
  }}
  
  encryption {{
    default_kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component     = "la-tgt-003"
    physical_node = "pa-cloud-001"
    layer         = "silver"
    environment   = var.environment
  }}
}}

resource "google_storage_bucket" "gold" {{
  name          = "${{var.project_name}}-gold-${{var.environment}}"
  location      = var.region
  force_destroy = false
  
  uniform_bucket_level_access = true
  
  versioning {{
    enabled = true
  }}
  
  encryption {{
    default_kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component     = "la-anlz-001"
    physical_node = "pa-anlz-001"
    layer         = "gold"
    environment   = var.environment
  }}
}}

resource "google_storage_bucket" "artifacts" {{
  name          = "${{var.project_name}}-artifacts-${{var.environment}}"
  location      = var.region
  force_destroy = false
  
  uniform_bucket_level_access = true
  
  labels = {{
    component   = "la-mig-001"
    environment = var.environment
  }}
}}

resource "google_firestore_database" "main" {{
  name        = "(default)"
  location_id = var.region
  type        = "FIRESTORE_NATIVE"
  
  depends_on = [google_project_service.firestore]
}}

resource "google_firestore_document" "migration_state" {{
  project     = var.project_id
  database    = google_firestore_database.main.name
  collection  = "migration_state"
  document_id = "config"
  
  fields = jsonencode({{
    environment = {{ stringValue = var.environment }}
    created     = {{ timestampValue = timestamp() }}
  }})
}}

resource "google_kms_key_ring" "main" {{
  name     = "${{var.project_name}}-keyring"
  location = var.region
}}

resource "google_kms_crypto_key" "storage" {{
  name            = "storage-key"
  key_ring        = google_kms_key_ring.main.id
  rotation_period = "7776000s"
  
  lifecycle {{
    prevent_destroy = true
  }}
}}

"#
    )
}

fn generate_gcp_compute_gke(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE - GOOGLE KUBERNETES ENGINE (GKE)
# ═══════════════════════════════════════════════════════════════════════════

resource "google_container_cluster" "main" {{
  name     = "${{var.project_name}}-gke"
  location = var.region
  
  remove_default_node_pool = true
  initial_node_count       = 1
  
  network    = google_compute_network.main.name
  subnetwork = google_compute_subnetwork.gke.name
  
  ip_allocation_policy {{
    cluster_secondary_range_name  = "gke-pods"
    services_secondary_range_name = "gke-services"
  }}
  
  workload_identity_config {{
    workload_pool = "${{var.project_id}}.svc.id.goog"
  }}
  
  addons_config {{
    horizontal_pod_autoscaling {{
      disabled = false
    }}
    
    http_load_balancing {{
      disabled = false
    }}
    
    gcp_filestore_csi_driver_config {{
      enabled = true
    }}
  }}
  
  release_channel {{
    channel = "REGULAR"
  }}
  
  binary_authorization {{
    evaluation_mode = "PROJECT_SINGLETON_POLICY_ENFORCE"
  }}
  
  monitoring_config {{
    enable_components = ["SYSTEM_COMPONENTS", "WORKLOADS"]
    
    managed_prometheus {{
      enabled = true
    }}
  }}
  
  resource_labels = {{
    component     = "la-proc-001"
    physical_node = "pa-dbx-002"
    requirement   = "sys-scale-001"
    environment   = var.environment
  }}
}}

resource "google_container_node_pool" "system" {{
  name       = "system-pool"
  location   = var.region
  cluster    = google_container_cluster.main.name
  node_count = 3
  
  autoscaling {{
    min_node_count = 3
    max_node_count = 10
  }}
  
  node_config {{
    machine_type = "e2-standard-4"
    disk_size_gb = 100
    disk_type    = "pd-standard"
    
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
    
    labels = {{
      pool = "system"
    }}
    
    workload_metadata_config {{
      mode = "GKE_METADATA"
    }}
    
    shielded_instance_config {{
      enable_secure_boot          = true
      enable_integrity_monitoring = true
    }}
  }}
  
  management {{
    auto_repair  = true
    auto_upgrade = true
  }}
}}

resource "google_container_node_pool" "data_processing" {{
  name       = "data-processing-pool"
  location   = var.region
  cluster    = google_container_cluster.main.name
  node_count = 2
  
  autoscaling {{
    min_node_count = 2
    max_node_count = 50
  }}
  
  node_config {{
    machine_type = "n2-highmem-8"
    disk_size_gb = 200
    disk_type    = "pd-ssd"
    
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
    
    labels = {{
      pool      = "data-processing"
      component = "la-proc-001"
    }}
    
    taint {{
      key    = "workload"
      value  = "data-processing"
      effect = "NO_SCHEDULE"
    }}
    
    workload_metadata_config {{
      mode = "GKE_METADATA"
    }}
    
    shielded_instance_config {{
      enable_secure_boot          = true
      enable_integrity_monitoring = true
    }}
  }}
  
  management {{
    auto_repair  = true
    auto_upgrade = true
  }}
}}

"#
    )
}

fn generate_gcp_bigquery(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# ANALYTICS - BIGQUERY
# ═══════════════════════════════════════════════════════════════════════════

resource "google_bigquery_dataset" "bronze" {{
  dataset_id    = "bronze"
  friendly_name = "Bronze Layer - Raw Data"
  description   = "Raw ingested data from source systems"
  location      = var.region
  
  default_table_expiration_ms = 7776000000
  
  default_encryption_configuration {{
    kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component = "la-tgt-003"
    layer     = "bronze"
  }}
}}

resource "google_bigquery_dataset" "silver" {{
  dataset_id    = "silver"
  friendly_name = "Silver Layer - Cleansed Data"
  description   = "Cleansed and validated data"
  location      = var.region
  
  default_encryption_configuration {{
    kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component = "la-proc-001"
    layer     = "silver"
  }}
}}

resource "google_bigquery_dataset" "gold" {{
  dataset_id    = "gold"
  friendly_name = "Gold Layer - Aggregated Data"
  description   = "Business-ready aggregated data"
  location      = var.region
  
  default_encryption_configuration {{
    kms_key_name = google_kms_crypto_key.storage.id
  }}
  
  labels = {{
    component = "la-anlz-001"
    layer     = "gold"
  }}
}}

resource "google_bigquery_table" "migration_state" {{
  dataset_id = google_bigquery_dataset.silver.dataset_id
  table_id   = "migration_state"
  
  deletion_protection = true
  
  time_partitioning {{
    type  = "DAY"
    field = "migration_date"
  }}
  
  schema = jsonencode([
    {{
      name = "table_name"
      type = "STRING"
      mode = "REQUIRED"
    }},
    {{
      name = "migration_date"
      type = "TIMESTAMP"
      mode = "REQUIRED"
    }},
    {{
      name = "status"
      type = "STRING"
      mode = "REQUIRED"
    }},
    {{
      name = "rows_migrated"
      type = "INTEGER"
      mode = "NULLABLE"
    }},
    {{
      name = "error_message"
      type = "STRING"
      mode = "NULLABLE"
    }}
  ])
  
  labels = {{
    component   = "la-mig-003"
    requirement = "sys-mig-003"
  }}
}}

resource "google_bigquery_routine" "data_quality_check" {{
  dataset_id   = google_bigquery_dataset.silver.dataset_id
  routine_id   = "data_quality_check"
  routine_type = "PROCEDURE"
  language     = "SQL"
  
  definition_body = <<-SQL
    BEGIN
      SELECT
        COUNT(*) as total_rows,
        COUNT(DISTINCT table_name) as tables_processed,
        COUNTIF(status = 'SUCCESS') as successful,
        COUNTIF(status = 'FAILED') as failed
      FROM `${{var.project_id}}.silver.migration_state`
      WHERE migration_date >= CURRENT_TIMESTAMP() - INTERVAL 24 HOUR;
    END;
  SQL
}}

"#
    )
}

fn generate_gcp_functions(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE - CLOUD FUNCTIONS (SERVERLESS)
# ═══════════════════════════════════════════════════════════════════════════

resource "google_cloudfunctions2_function" "schema_converter" {{
  name     = "${{var.project_name}}-schema-converter"
  location = var.region
  
  build_config {{
    runtime     = "python311"
    entry_point = "convert_schema"
    
    source {{
      storage_source {{
        bucket = google_storage_bucket.artifacts.name
        object = "schema-converter.zip"
      }}
    }}
  }}
  
  service_config {{
    max_instance_count    = 100
    min_instance_count    = 1
    available_memory      = "4Gi"
    timeout_seconds       = 540
    max_instance_request_concurrency = 1
    
    environment_variables = {{
      ENVIRONMENT = var.environment
      PROJECT_ID  = var.project_id
    }}
    
    ingress_settings = "ALLOW_INTERNAL_ONLY"
    
    service_account_email = google_service_account.functions.email
  }}
  
  labels = {{
    component     = "la-mig-002"
    physical_node = "pa-mig-001"
    requirement   = "sys-mig-002"
  }}
}}

resource "google_cloudfunctions2_function" "data_validator" {{
  name     = "${{var.project_name}}-data-validator"
  location = var.region
  
  build_config {{
    runtime     = "python311"
    entry_point = "validate_data"
    
    source {{
      storage_source {{
        bucket = google_storage_bucket.artifacts.name
        object = "data-validator.zip"
      }}
    }}
  }}
  
  service_config {{
    max_instance_count = 100
    available_memory   = "4Gi"
    timeout_seconds    = 540
    
    environment_variables = {{
      ENVIRONMENT = var.environment
      PROJECT_ID  = var.project_id
    }}
    
    ingress_settings      = "ALLOW_INTERNAL_ONLY"
    service_account_email = google_service_account.functions.email
  }}
  
  labels = {{
    component     = "la-mig-003"
    physical_node = "pa-mig-001"
    requirement   = "sys-mig-003"
    safety_level  = "critical"
  }}
}}

resource "google_service_account" "functions" {{
  account_id   = "${{var.project_name}}-functions"
  display_name = "Cloud Functions Service Account"
}}

resource "google_project_iam_member" "functions_storage" {{
  project = var.project_id
  role    = "roles/storage.objectViewer"
  member  = "serviceAccount:${{google_service_account.functions.email}}"
}}

resource "google_project_iam_member" "functions_bigquery" {{
  project = var.project_id
  role    = "roles/bigquery.dataEditor"
  member  = "serviceAccount:${{google_service_account.functions.email}}"
}}

"#
    )
}

fn generate_gcp_integration(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# INTEGRATION - PUB/SUB, CLOUD TASKS, API GATEWAY
# ═══════════════════════════════════════════════════════════════════════════

resource "google_pubsub_topic" "migration_events" {{
  name = "migration-events"
  
  message_retention_duration = "604800s"
  
  labels = {{
    component     = "la-int-003"
    physical_node = "pa-int-001"
  }}
}}

resource "google_pubsub_topic" "data_quality_alerts" {{
  name = "data-quality-alerts"
  
  message_retention_duration = "86400s"
  
  labels = {{
    component   = "la-proc-003"
    requirement = "sys-qual-001"
  }}
}}

resource "google_pubsub_subscription" "migration_events_sub" {{
  name  = "migration-events-sub"
  topic = google_pubsub_topic.migration_events.name
  
  ack_deadline_seconds = 600
  
  retry_policy {{
    minimum_backoff = "10s"
    maximum_backoff = "600s"
  }}
  
  dead_letter_policy {{
    dead_letter_topic     = google_pubsub_topic.dead_letter.id
    max_delivery_attempts = 5
  }}
  
  expiration_policy {{
    ttl = ""
  }}
}}

resource "google_pubsub_topic" "dead_letter" {{
  name = "dead-letter-topic"
}}

resource "google_cloud_tasks_queue" "etl_jobs" {{
  name     = "etl-jobs"
  location = var.region
  
  rate_limits {{
    max_concurrent_dispatches = 100
    max_dispatches_per_second = 50
  }}
  
  retry_config {{
    max_attempts       = 5
    max_retry_duration = "3600s"
    min_backoff        = "10s"
    max_backoff        = "600s"
    max_doublings      = 3
  }}
}}

resource "google_api_gateway_api" "main" {{
  provider     = google-beta
  api_id       = "${{var.project_name}}-api"
  display_name = "Data Platform API"
}}

resource "google_api_gateway_api_config" "main" {{
  provider      = google-beta
  api           = google_api_gateway_api.main.api_id
  api_config_id = "${{var.project_name}}-config"
  
  openapi_documents {{
    document {{
      path = "openapi.yaml"
      contents = base64encode(<<-EOT
        openapi: 3.0.0
        info:
          title: Data Platform API
          version: 1.0.0
        paths:
          /migration-jobs:
            post:
              summary: Trigger migration job
              operationId: triggerMigration
              responses:
                '200':
                  description: Success
      EOT
      )
    }}
  }}
  
  lifecycle {{
    create_before_destroy = true
  }}
}}

resource "google_api_gateway_gateway" "main" {{
  provider   = google-beta
  api_config = google_api_gateway_api_config.main.id
  gateway_id = "${{var.project_name}}-gateway"
  region     = var.region
  
  labels = {{
    component     = "la-int-001"
    physical_node = "pa-int-001"
  }}
}}

"#
    )
}

fn generate_gcp_orchestration(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# ORCHESTRATION - CLOUD COMPOSER (AIRFLOW), WORKFLOWS
# ═══════════════════════════════════════════════════════════════════════════

resource "google_workflows_workflow" "etl_pipeline" {{
  name            = "etl-pipeline"
  region          = var.region
  description     = "Main ETL pipeline workflow"
  service_account = google_service_account.workflows.email
  
  source_contents = <<-EOT
    main:
      steps:
        - extract_parallel:
            parallel:
              branches:
                - extract_oracle:
                    call: http.post
                    args:
                      url: https://us-central1-${{var.project_id}}.cloudfunctions.net/extract-oracle
                - extract_snowflake:
                    call: http.post
                    args:
                      url: https://us-central1-${{var.project_id}}.cloudfunctions.net/extract-snowflake
        - schema_conversion:
            call: googleapis.cloudfunctions.v2.projects.locations.functions.call
            args:
              name: ${{google_cloudfunctions2_function.schema_converter.name}}
        - data_validation:
            call: googleapis.cloudfunctions.v2.projects.locations.functions.call
            args:
              name: ${{google_cloudfunctions2_function.data_validator.name}}
        - publish_success:
            call: googleapis.pubsub.v1.projects.topics.publish
            args:
              topic: ${{google_pubsub_topic.migration_events.id}}
              messages:
                - data: ${{base64.encode(json.encode(result))}}
  EOT
  
  labels = {{
    component     = "la-mig-001"
    physical_node = "pa-mig-001"
    requirement   = "sys-mig-005"
  }}
}}

resource "google_cloud_scheduler_job" "daily_etl" {{
  name             = "daily-etl-trigger"
  description      = "Daily ETL pipeline trigger"
  schedule         = "0 2 * * *"
  time_zone        = "America/New_York"
  attempt_deadline = "1800s"
  region           = var.region
  
  http_target {{
    http_method = "POST"
    uri         = "https://workflowexecutions.googleapis.com/v1/${{google_workflows_workflow.etl_pipeline.id}}/executions"
    
    oauth_token {{
      service_account_email = google_service_account.workflows.email
    }}
  }}
}}

resource "google_service_account" "workflows" {{
  account_id   = "${{var.project_name}}-workflows"
  display_name = "Workflows Service Account"
}}

resource "google_project_iam_member" "workflows_invoker" {{
  project = var.project_id
  role    = "roles/workflows.invoker"
  member  = "serviceAccount:${{google_service_account.workflows.email}}"
}}

resource "google_project_iam_member" "workflows_functions" {{
  project = var.project_id
  role    = "roles/cloudfunctions.invoker"
  member  = "serviceAccount:${{google_service_account.workflows.email}}"
}}

"#
    )
}

fn generate_gcp_monitoring(_model: &SemanticModel, config: &GcpConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# MONITORING - CLOUD MONITORING, LOGGING, ALERTING
# ═══════════════════════════════════════════════════════════════════════════

resource "google_monitoring_alert_policy" "workflow_failures" {{
  display_name = "Workflow Execution Failures"
  combiner     = "OR"
  
  conditions {{
    display_name = "Workflow failed"
    
    condition_threshold {{
      filter          = "resource.type=\"workflows.googleapis.com/Workflow\" AND metric.type=\"workflows.googleapis.com/workflow/execution/error_count\""
      duration        = "60s"
      comparison      = "COMPARISON_GT"
      threshold_value = 0
      
      aggregations {{
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_SUM"
      }}
    }}
  }}
  
  notification_channels = [google_monitoring_notification_channel.email.name]
  
  alert_strategy {{
    auto_close = "86400s"
  }}
}}

resource "google_monitoring_alert_policy" "function_errors" {{
  display_name = "Cloud Function Errors"
  combiner     = "OR"
  
  conditions {{
    display_name = "Function error rate high"
    
    condition_threshold {{
      filter          = "resource.type=\"cloud_function\" AND metric.type=\"cloudfunctions.googleapis.com/function/execution_count\" AND metric.label.status=\"error\""
      duration        = "300s"
      comparison      = "COMPARISON_GT"
      threshold_value = 5
      
      aggregations {{
        alignment_period   = "300s"
        per_series_aligner = "ALIGN_RATE"
      }}
    }}
  }}
  
  notification_channels = [google_monitoring_notification_channel.email.name]
}}

resource "google_monitoring_alert_policy" "gke_node_cpu" {{
  display_name = "GKE Node High CPU Usage"
  combiner     = "OR"
  
  conditions {{
    display_name = "CPU usage > 80%"
    
    condition_threshold {{
      filter          = "resource.type=\"k8s_node\" AND metric.type=\"kubernetes.io/node/cpu/allocatable_utilization\""
      duration        = "300s"
      comparison      = "COMPARISON_GT"
      threshold_value = 0.8
      
      aggregations {{
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_MEAN"
      }}
    }}
  }}
  
  notification_channels = [google_monitoring_notification_channel.email.name]
}}

resource "google_monitoring_notification_channel" "email" {{
  display_name = "Data Engineering Team"
  type         = "email"
  
  labels = {{
    email_address = "data-engineering@company.com"
  }}
}}

resource "google_monitoring_dashboard" "main" {{
  dashboard_json = jsonencode({{
    displayName = "Data Platform Dashboard"
    
    mosaicLayout = {{
      columns = 12
      tiles = [
        {{
          width  = 6
          height = 4
          widget = {{
            title = "Workflow Executions"
            xyChart = {{
              dataSets = [{{
                timeSeriesQuery = {{
                  timeSeriesFilter = {{
                    filter = "resource.type=\"workflows.googleapis.com/Workflow\" AND metric.type=\"workflows.googleapis.com/workflow/execution/success_count\""
                    aggregation = {{
                      alignmentPeriod  = "60s"
                      perSeriesAligner = "ALIGN_RATE"
                    }}
                  }}
                }}
                plotType = "LINE"
              }}]
            }}
          }}
        }},
        {{
          width  = 6
          height = 4
          widget = {{
            title = "Function Invocations"
            xyChart = {{
              dataSets = [{{
                timeSeriesQuery = {{
                  timeSeriesFilter = {{
                    filter = "resource.type=\"cloud_function\" AND metric.type=\"cloudfunctions.googleapis.com/function/execution_count\""
                    aggregation = {{
                      alignmentPeriod  = "60s"
                      perSeriesAligner = "ALIGN_RATE"
                    }}
                  }}
                }}
                plotType = "LINE"
              }}]
            }}
          }}
        }}
      ]
    }}
  }})
  
  lifecycle {{
    ignore_changes = [dashboard_json]
  }}
}}

resource "google_logging_project_sink" "bigquery_sink" {{
  name        = "bigquery-audit-sink"
  destination = "bigquery.googleapis.com/projects/${{var.project_id}}/datasets/${{google_bigquery_dataset.silver.dataset_id}}"
  
  filter = <<-EOT
    resource.type="cloud_function" OR
    resource.type="k8s_cluster" OR
    resource.type="workflows.googleapis.com/Workflow"
  EOT
  
  unique_writer_identity = true
}}

"#
    )
}

fn generate_gcp_outputs(model: &SemanticModel) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# OUTPUTS
# ═══════════════════════════════════════════════════════════════════════════

output "gke_cluster_name" {{
  description = "GKE cluster name"
  value       = google_container_cluster.main.name
}}

output "gke_endpoint" {{
  description = "GKE cluster endpoint"
  value       = google_container_cluster.main.endpoint
  sensitive   = true
}}

output "bronze_bucket" {{
  description = "Bronze layer storage bucket"
  value       = google_storage_bucket.bronze.name
}}

output "silver_bucket" {{
  description = "Silver layer storage bucket"
  value       = google_storage_bucket.silver.name
}}

output "gold_bucket" {{
  description = "Gold layer storage bucket"
  value       = google_storage_bucket.gold.name
}}

output "bigquery_bronze_dataset" {{
  description = "BigQuery bronze dataset"
  value       = google_bigquery_dataset.bronze.dataset_id
}}

output "bigquery_silver_dataset" {{
  description = "BigQuery silver dataset"
  value       = google_bigquery_dataset.silver.dataset_id
}}

output "bigquery_gold_dataset" {{
  description = "BigQuery gold dataset"
  value       = google_bigquery_dataset.gold.dataset_id
}}

output "api_gateway_url" {{
  description = "API Gateway URL"
  value       = google_api_gateway_gateway.main.default_hostname
}}

output "workflow_id" {{
  description = "ETL workflow ID"
  value       = google_workflows_workflow.etl_pipeline.id
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
