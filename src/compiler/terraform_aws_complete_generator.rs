// ═══════════════════════════════════════════════════════════════════════════
// COMPLETE AWS TERRAFORM GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate complete AWS infrastructure from ArcLang physical architecture
// Coverage: VPC, EC2, ECS, Lambda, Step Functions, RDS, S3, API Gateway, EventBridge
// Target: AWS Provider v5.0+ (2025), Databricks v1.86+
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, ComponentInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_complete_aws_terraform(
    model: &SemanticModel,
    config: &AwsConfig,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    
    // Main terraform configuration
    output.push_str(&generate_terraform_header(config));
    output.push_str("\n");
    
    // Provider configuration
    output.push_str(&generate_providers(config));
    output.push_str("\n");
    
    // Variables
    output.push_str(&generate_variables());
    output.push_str("\n");
    
    // 1. NETWORKING LAYER - VPC, Subnets, Security Groups
    output.push_str(&generate_networking_layer(model, config));
    output.push_str("\n");
    
    // 2. DATA LAYER - S3, RDS, DynamoDB, ElastiCache
    output.push_str(&generate_data_layer(model, config));
    output.push_str("\n");
    
    // 3. COMPUTE LAYER - EC2, ECS, Lambda, Databricks
    output.push_str(&generate_compute_layer(model, config));
    output.push_str("\n");
    
    // 4. INTEGRATION LAYER - API Gateway, EventBridge, SNS, SQS
    output.push_str(&generate_integration_layer(model, config));
    output.push_str("\n");
    
    // 5. ORCHESTRATION LAYER - Step Functions, Glue, MWAA (Airflow)
    output.push_str(&generate_orchestration_layer(model, config));
    output.push_str("\n");
    
    // 6. GOVERNANCE LAYER - IAM, KMS, Secrets Manager, Config
    output.push_str(&generate_governance_layer(model, config));
    output.push_str("\n");
    
    // 7. MONITORING LAYER - CloudWatch, X-Ray, OpenTelemetry
    output.push_str(&generate_monitoring_layer(model, config));
    output.push_str("\n");
    
    // 8. ANALYTICS LAYER - Athena, QuickSight, Redshift
    output.push_str(&generate_analytics_layer(model, config));
    output.push_str("\n");
    
    // Outputs
    output.push_str(&generate_outputs(model));
    
    Ok(output)
}

#[derive(Debug, Clone)]
pub struct AwsConfig {
    pub region: String,
    pub account_id: String,
    pub environment: String,
    pub project_name: String,
    pub vpc_cidr: String,
    pub availability_zones: Vec<String>,
    pub enable_nat_gateway: bool,
    pub enable_vpn_gateway: bool,
    pub tags: HashMap<String, String>,
}

impl Default for AwsConfig {
    fn default() -> Self {
        let mut tags = HashMap::new();
        tags.insert("Project".to_string(), "DataPlatformMigration".to_string());
        tags.insert("ManagedBy".to_string(), "Terraform".to_string());
        tags.insert("GeneratedFrom".to_string(), "ArcLang".to_string());
        
        AwsConfig {
            region: "us-east-1".to_string(),
            account_id: "123456789012".to_string(),
            environment: "prod".to_string(),
            project_name: "data-platform".to_string(),
            vpc_cidr: "10.0.0.0/16".to_string(),
            availability_zones: vec!["us-east-1a".to_string(), "us-east-1b".to_string(), "us-east-1c".to_string()],
            enable_nat_gateway: true,
            enable_vpn_gateway: false,
            tags,
        }
    }
}

fn generate_terraform_header(config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPLETE AWS INFRASTRUCTURE - GENERATED FROM ARCLANG MODEL
# ═══════════════════════════════════════════════════════════════════════════
# Generated: {}
# Region: {}
# Environment: {}
# Project: {}
# ═══════════════════════════════════════════════════════════════════════════

terraform {{
  required_version = ">= 1.5.0"
  
  required_providers {{
    aws = {{
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }}
    databricks = {{
      source  = "databricks/databricks"
      version = "~> 1.86"
    }}
    random = {{
      source  = "hashicorp/random"
      version = "~> 3.5"
    }}
  }}
  
  backend "s3" {{
    bucket         = "{}-terraform-state"
    key            = "{}/terraform.tfstate"
    region         = "{}"
    encrypt        = true
    dynamodb_table = "{}-terraform-locks"
  }}
}}
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.region,
        config.environment,
        config.project_name,
        config.project_name,
        config.environment,
        config.region,
        config.project_name
    )
}

fn generate_providers(config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# PROVIDER CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

provider "aws" {{
  region = var.aws_region
  
  default_tags {{
    tags = {{
      Environment = var.environment
      Project     = var.project_name
      ManagedBy   = "Terraform"
      GeneratedFrom = "ArcLang"
    }}
  }}
}}

provider "databricks" {{
  host = var.databricks_workspace_url
}}

provider "random" {{}}
"#
    )
}

fn generate_variables() -> String {
r#"# ═══════════════════════════════════════════════════════════════════════════
# VARIABLES
# ═══════════════════════════════════════════════════════════════════════════

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "prod"
}

variable "project_name" {
  description = "Project name for resource naming"
  type        = string
  default     = "data-platform"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of availability zones"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "databricks_workspace_url" {
  description = "Databricks workspace URL"
  type        = string
}

variable "oracle_db_endpoint" {
  description = "Oracle database endpoint"
  type        = string
}

variable "snowflake_account" {
  description = "Snowflake account identifier"
  type        = string
}

"#.to_string()
}

fn generate_networking_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# NETWORKING LAYER - VPC, SUBNETS, SECURITY GROUPS
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-CLOUD-001 (AWS Cloud Infrastructure)
# Physical Link: PL-001 (AWS VPC Primary Network)
# ═══════════════════════════════════════════════════════════════════════════

# VPC
resource "aws_vpc" "main" {{
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {{
    Name = "${{var.project_name}}-vpc"
    PhysicalNode = "PA-CLOUD-001"
  }}
}}

# Internet Gateway
resource "aws_internet_gateway" "main" {{
  vpc_id = aws_vpc.main.id
  
  tags = {{
    Name = "${{var.project_name}}-igw"
  }}
}}

# Public Subnets
resource "aws_subnet" "public" {{
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index)
  availability_zone = var.availability_zones[count.index]
  
  map_public_ip_on_launch = true
  
  tags = {{
    Name = "${{var.project_name}}-public-${{count.index + 1}}"
    Type = "public"
  }}
}}

# Private Subnets (for data layer)
resource "aws_subnet" "private_data" {{
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 10)
  availability_zone = var.availability_zones[count.index]
  
  tags = {{
    Name = "${{var.project_name}}-private-data-${{count.index + 1}}"
    Type = "private_data"
  }}
}}

# Private Subnets (for compute layer)
resource "aws_subnet" "private_compute" {{
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 20)
  availability_zone = var.availability_zones[count.index]
  
  tags = {{
    Name = "${{var.project_name}}-private-compute-${{count.index + 1}}"
    Type = "private_compute"
  }}
}}

# NAT Gateways
resource "aws_eip" "nat" {{
  count  = length(var.availability_zones)
  domain = "vpc"
  
  tags = {{
    Name = "${{var.project_name}}-nat-eip-${{count.index + 1}}"
  }}
}}

resource "aws_nat_gateway" "main" {{
  count         = length(var.availability_zones)
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  
  tags = {{
    Name = "${{var.project_name}}-nat-${{count.index + 1}}"
  }}
  
  depends_on = [aws_internet_gateway.main]
}}

# Route Tables
resource "aws_route_table" "public" {{
  vpc_id = aws_vpc.main.id
  
  route {{
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }}
  
  tags = {{
    Name = "${{var.project_name}}-public-rt"
  }}
}}

resource "aws_route_table" "private" {{
  count  = length(var.availability_zones)
  vpc_id = aws_vpc.main.id
  
  route {{
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }}
  
  tags = {{
    Name = "${{var.project_name}}-private-rt-${{count.index + 1}}"
  }}
}}

# Route Table Associations
resource "aws_route_table_association" "public" {{
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}}

resource "aws_route_table_association" "private_data" {{
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private_data[count.index].id
  route_table_id = aws_route_table.private[count.index].id
}}

resource "aws_route_table_association" "private_compute" {{
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private_compute[count.index].id
  route_table_id = aws_route_table.private[count.index].id
}}

# Security Group - Migration Engine
resource "aws_security_group" "migration_engine" {{
  name        = "${{var.project_name}}-migration-engine-sg"
  description = "Security group for migration engine components"
  vpc_id      = aws_vpc.main.id
  
  ingress {{
    description = "Oracle connection"
    from_port   = 1521
    to_port     = 1521
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }}
  
  ingress {{
    description = "Snowflake connection"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }}
  
  egress {{
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }}
  
  tags = {{
    Name = "${{var.project_name}}-migration-engine-sg"
    Component = "LA-MIG-001"
    PhysicalNode = "PA-MIG-001"
  }}
}}

# Security Group - Data Processing
resource "aws_security_group" "data_processing" {{
  name        = "${{var.project_name}}-data-processing-sg"
  description = "Security group for data processing components"
  vpc_id      = aws_vpc.main.id
  
  ingress {{
    description = "Internal communication"
    from_port   = 0
    to_port     = 65535
    protocol    = "tcp"
    self        = true
  }}
  
  egress {{
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }}
  
  tags = {{
    Name = "${{var.project_name}}-data-processing-sg"
    Component = "LA-PROC-001"
    PhysicalNode = "PA-DBX-002"
  }}
}}

# VPC Endpoints for AWS Services
resource "aws_vpc_endpoint" "s3" {{
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${{var.aws_region}}.s3"
  
  route_table_ids = concat(
    [aws_route_table.public.id],
    aws_route_table.private[*].id
  )
  
  tags = {{
    Name = "${{var.project_name}}-s3-endpoint"
  }}
}}

resource "aws_vpc_endpoint" "dynamodb" {{
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${{var.aws_region}}.dynamodb"
  
  route_table_ids = concat(
    [aws_route_table.public.id],
    aws_route_table.private[*].id
  )
  
  tags = {{
    Name = "${{var.project_name}}-dynamodb-endpoint"
  }}
}}

"#
    )
}

fn generate_data_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# DATA LAYER - S3, RDS, DYNAMODB, ELASTICACHE
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-CLOUD-001 (Delta Lake Storage - LA-TGT-003)
# Physical Node: PA-SRC-001 (Source System Gateway - LA-SRC-001, LA-SRC-002)
# ═══════════════════════════════════════════════════════════════════════════

# S3 Bucket - Delta Lake Storage (Bronze)
resource "aws_s3_bucket" "delta_lake_bronze" {{
  bucket = "${{var.project_name}}-delta-lake-bronze-${{var.environment}}"
  
  tags = {{
    Name = "Delta Lake Bronze Layer"
    Component = "LA-TGT-003"
    PhysicalNode = "PA-CLOUD-001"
    Layer = "bronze"
    Requirement = "SYS-PERF-002"
  }}
}}

resource "aws_s3_bucket_versioning" "delta_lake_bronze" {{
  bucket = aws_s3_bucket.delta_lake_bronze.id
  
  versioning_configuration {{
    status = "Enabled"
  }}
}}

resource "aws_s3_bucket_server_side_encryption_configuration" "delta_lake_bronze" {{
  bucket = aws_s3_bucket.delta_lake_bronze.id
  
  rule {{
    apply_server_side_encryption_by_default {{
      sse_algorithm     = "aws:kms"
      kms_master_key_id = aws_kms_key.data_encryption.arn
    }}
  }}
}}

resource "aws_s3_bucket_lifecycle_configuration" "delta_lake_bronze" {{
  bucket = aws_s3_bucket.delta_lake_bronze.id
  
  rule {{
    id     = "archive_old_data"
    status = "Enabled"
    
    transition {{
      days          = 90
      storage_class = "GLACIER_IR"
    }}
    
    transition {{
      days          = 180
      storage_class = "DEEP_ARCHIVE"
    }}
  }}
}}

# S3 Bucket - Delta Lake Storage (Silver)
resource "aws_s3_bucket" "delta_lake_silver" {{
  bucket = "${{var.project_name}}-delta-lake-silver-${{var.environment}}"
  
  tags = {{
    Name = "Delta Lake Silver Layer"
    Component = "LA-PROC-001"
    Layer = "silver"
  }}
}}

resource "aws_s3_bucket_versioning" "delta_lake_silver" {{
  bucket = aws_s3_bucket.delta_lake_silver.id
  
  versioning_configuration {{
    status = "Enabled"
  }}
}}

# S3 Bucket - Delta Lake Storage (Gold)
resource "aws_s3_bucket" "delta_lake_gold" {{
  bucket = "${{var.project_name}}-delta-lake-gold-${{var.environment}}"
  
  tags = {{
    Name = "Delta Lake Gold Layer"
    Component = "LA-ANLZ-001"
    Layer = "gold"
  }}
}}

# S3 Bucket - ETL Artifacts
resource "aws_s3_bucket" "etl_artifacts" {{
  bucket = "${{var.project_name}}-etl-artifacts-${{var.environment}}"
  
  tags = {{
    Name = "ETL Artifacts Bucket"
    Component = "LA-MIG-001"
    PhysicalNode = "PA-MIG-001"
  }}
}}

# DynamoDB Table - Migration State Tracking
resource "aws_dynamodb_table" "migration_state" {{
  name           = "${{var.project_name}}-migration-state"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "table_name"
  range_key      = "migration_timestamp"
  
  attribute {{
    name = "table_name"
    type = "S"
  }}
  
  attribute {{
    name = "migration_timestamp"
    type = "N"
  }}
  
  attribute {{
    name = "status"
    type = "S"
  }}
  
  global_secondary_index {{
    name            = "StatusIndex"
    hash_key        = "status"
    range_key       = "migration_timestamp"
    projection_type = "ALL"
  }}
  
  point_in_time_recovery {{
    enabled = true
  }}
  
  server_side_encryption {{
    enabled     = true
    kms_key_arn = aws_kms_key.data_encryption.arn
  }}
  
  tags = {{
    Name = "Migration State Tracker"
    Component = "LA-MIG-003"
    Requirement = "SYS-MIG-003"
  }}
}}

# DynamoDB Table - Data Validation Results
resource "aws_dynamodb_table" "validation_results" {{
  name           = "${{var.project_name}}-validation-results"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "validation_id"
  
  attribute {{
    name = "validation_id"
    type = "S"
  }}
  
  attribute {{
    name = "validation_timestamp"
    type = "N"
  }}
  
  global_secondary_index {{
    name            = "TimestampIndex"
    hash_key        = "validation_timestamp"
    projection_type = "ALL"
  }}
  
  point_in_time_recovery {{
    enabled = true
  }}
  
  ttl {{
    attribute_name = "expiration_time"
    enabled        = true
  }}
  
  tags = {{
    Name = "Validation Results"
    Component = "LA-MIG-003"
    SafetyLevel = "Critical"
  }}
}}

# ElastiCache - Redis for Caching
resource "aws_elasticache_subnet_group" "main" {{
  name       = "${{var.project_name}}-cache-subnet"
  subnet_ids = aws_subnet.private_data[*].id
  
  tags = {{
    Name = "ElastiCache Subnet Group"
  }}
}}

resource "aws_elasticache_replication_group" "metadata_cache" {{
  replication_group_id       = "${{var.project_name}}-metadata-cache"
  replication_group_description = "Redis cache for metadata and query results"
  engine                     = "redis"
  engine_version             = "7.0"
  node_type                  = "cache.r7g.large"
  number_cache_clusters      = 3
  port                       = 6379
  parameter_group_name       = "default.redis7"
  subnet_group_name          = aws_elasticache_subnet_group.main.name
  security_group_ids         = [aws_security_group.data_processing.id]
  automatic_failover_enabled = true
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  
  tags = {{
    Name = "Metadata Cache"
    Component = "LA-INT-002"
    Requirement = "SYS-PERF-001"
  }}
}}

"#
    )
}

fn generate_compute_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COMPUTE LAYER - ECS, LAMBDA, EC2
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-MIG-001 (Migration Engine Node)
# Physical Node: PA-DBX-002 (Databricks Workspace Cluster)
# Physical Node: PA-INT-001 (Integration Gateway Node)
# ═══════════════════════════════════════════════════════════════════════════

# ECS Cluster - Migration Engine
resource "aws_ecs_cluster" "migration_engine" {{
  name = "${{var.project_name}}-migration-engine"
  
  setting {{
    name  = "containerInsights"
    value = "enabled"
  }}
  
  tags = {{
    Name = "Migration Engine Cluster"
    Component = "LA-MIG-001"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-005"
  }}
}}

resource "aws_ecs_cluster_capacity_providers" "migration_engine" {{
  cluster_name = aws_ecs_cluster.migration_engine.name
  
  capacity_providers = ["FARGATE", "FARGATE_SPOT"]
  
  default_capacity_provider_strategy {{
    capacity_provider = "FARGATE_SPOT"
    weight            = 4
    base              = 0
  }}
  
  default_capacity_provider_strategy {{
    capacity_provider = "FARGATE"
    weight            = 1
    base              = 1
  }}
}}

# ECS Task Definition - ETL Orchestrator
resource "aws_ecs_task_definition" "etl_orchestrator" {{
  family                   = "${{var.project_name}}-etl-orchestrator"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "8192"
  memory                   = "65536"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.etl_task.arn
  
  container_definitions = jsonencode([{{
    name      = "etl-orchestrator"
    image     = "${{var.aws_region}}.dkr.ecr.${{var.aws_region}}.amazonaws.com/${{var.project_name}}/etl-orchestrator:latest"
    cpu       = 8192
    memory    = 65536
    essential = true
    
    environment = [
      {{
        name  = "ENVIRONMENT"
        value = var.environment
      }},
      {{
        name  = "S3_BUCKET"
        value = aws_s3_bucket.delta_lake_bronze.bucket
      }}
    ]
    
    logConfiguration = {{
      logDriver = "awslogs"
      options = {{
        "awslogs-group"         = "/ecs/${{var.project_name}}/etl-orchestrator"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }}
    }}
    
    healthCheck = {{
      command     = ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }}
  }}])
  
  tags = {{
    Name = "ETL Orchestrator Task"
    Component = "LA-MIG-001"
    PhysicalNode = "PA-MIG-001"
  }}
}}

# ECS Service - ETL Orchestrator
resource "aws_ecs_service" "etl_orchestrator" {{
  name            = "${{var.project_name}}-etl-orchestrator"
  cluster         = aws_ecs_cluster.migration_engine.id
  task_definition = aws_ecs_task_definition.etl_orchestrator.arn
  desired_count   = 2
  launch_type     = "FARGATE"
  
  network_configuration {{
    subnets          = aws_subnet.private_compute[*].id
    security_groups  = [aws_security_group.migration_engine.id]
    assign_public_ip = false
  }}
  
  service_registries {{
    registry_arn = aws_service_discovery_service.etl_orchestrator.arn
  }}
  
  tags = {{
    Name = "ETL Orchestrator Service"
    Component = "LA-MIG-001"
  }}
}}

# Lambda Function - Schema Converter
resource "aws_lambda_function" "schema_converter" {{
  function_name = "${{var.project_name}}-schema-converter"
  role          = aws_iam_role.lambda_execution.arn
  runtime       = "python3.12"
  handler       = "lambda_function.lambda_handler"
  timeout       = 900
  memory_size   = 10240
  
  filename         = "${{path.module}}/lambda/schema_converter.zip"
  source_code_hash = filebase64sha256("${{path.module}}/lambda/schema_converter.zip")
  
  environment {{
    variables = {{
      ENVIRONMENT       = var.environment
      S3_BUCKET         = aws_s3_bucket.delta_lake_bronze.bucket
      DYNAMODB_TABLE    = aws_dynamodb_table.migration_state.name
    }}
  }}
  
  vpc_config {{
    subnet_ids         = aws_subnet.private_compute[*].id
    security_group_ids = [aws_security_group.migration_engine.id]
  }}
  
  tracing_config {{
    mode = "Active"
  }}
  
  tags = {{
    Name = "Schema Converter"
    Component = "LA-MIG-002"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-002"
  }}
}}

# Lambda Function - Data Validator
resource "aws_lambda_function" "data_validator" {{
  function_name = "${{var.project_name}}-data-validator"
  role          = aws_iam_role.lambda_execution.arn
  runtime       = "python3.12"
  handler       = "lambda_function.lambda_handler"
  timeout       = 900
  memory_size   = 10240
  
  filename         = "${{path.module}}/lambda/data_validator.zip"
  source_code_hash = filebase64sha256("${{path.module}}/lambda/data_validator.zip")
  
  environment {{
    variables = {{
      ENVIRONMENT       = var.environment
      VALIDATION_TABLE  = aws_dynamodb_table.validation_results.name
      SNS_TOPIC_ARN     = aws_sns_topic.data_quality_alerts.arn
    }}
  }}
  
  vpc_config {{
    subnet_ids         = aws_subnet.private_compute[*].id
    security_group_ids = [aws_security_group.migration_engine.id]
  }}
  
  tracing_config {{
    mode = "Active"
  }}
  
  tags = {{
    Name = "Data Validator"
    Component = "LA-MIG-003"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-003"
    SafetyLevel = "Critical"
  }}
}}

# Lambda Function - Conflict Resolver
resource "aws_lambda_function" "conflict_resolver" {{
  function_name = "${{var.project_name}}-conflict-resolver"
  role          = aws_iam_role.lambda_execution.arn
  runtime       = "python3.12"
  handler       = "lambda_function.lambda_handler"
  timeout       = 300
  memory_size   = 3008
  
  filename         = "${{path.module}}/lambda/conflict_resolver.zip"
  source_code_hash = filebase64sha256("${{path.module}}/lambda/conflict_resolver.zip")
  
  environment {{
    variables = {{
      ENVIRONMENT    = var.environment
      STATE_TABLE    = aws_dynamodb_table.migration_state.name
      REDIS_ENDPOINT = aws_elasticache_replication_group.metadata_cache.primary_endpoint_address
    }}
  }}
  
  vpc_config {{
    subnet_ids         = aws_subnet.private_compute[*].id
    security_group_ids = [aws_security_group.migration_engine.id]
  }}
  
  tags = {{
    Name = "Conflict Resolver"
    Component = "LA-MIG-004"
    Requirement = "SYS-MIG-001"
  }}
}}

# Lambda Function - Streaming Data Processor
resource "aws_lambda_function" "streaming_processor" {{
  function_name = "${{var.project_name}}-streaming-processor"
  role          = aws_iam_role.lambda_execution.arn
  runtime       = "python3.12"
  handler       = "lambda_function.lambda_handler"
  timeout       = 60
  memory_size   = 1024
  
  filename         = "${{path.module}}/lambda/streaming_processor.zip"
  source_code_hash = filebase64sha256("${{path.module}}/lambda/streaming_processor.zip")
  
  environment {{
    variables = {{
      ENVIRONMENT   = var.environment
      S3_BUCKET     = aws_s3_bucket.delta_lake_bronze.bucket
      KINESIS_STREAM = aws_kinesis_stream.data_ingestion.name
    }}
  }}
  
  tags = {{
    Name = "Streaming Data Processor"
    Component = "LA-PROC-002"
    Requirement = "SYS-PERF-004"
  }}
}}

# Lambda Event Source Mapping - Kinesis
resource "aws_lambda_event_source_mapping" "kinesis" {{
  event_source_arn  = aws_kinesis_stream.data_ingestion.arn
  function_name     = aws_lambda_function.streaming_processor.arn
  starting_position = "LATEST"
  
  batch_size                         = 100
  maximum_batching_window_in_seconds = 10
  parallelization_factor             = 10
  
  destination_config {{
    on_failure {{
      destination_arn = aws_sqs_queue.streaming_dlq.arn
    }}
  }}
}}

# Service Discovery
resource "aws_service_discovery_private_dns_namespace" "main" {{
  name        = "${{var.project_name}}.local"
  description = "Private DNS namespace for service discovery"
  vpc         = aws_vpc.main.id
}}

resource "aws_service_discovery_service" "etl_orchestrator" {{
  name = "etl-orchestrator"
  
  dns_config {{
    namespace_id = aws_service_discovery_private_dns_namespace.main.id
    
    dns_records {{
      ttl  = 10
      type = "A"
    }}
    
    routing_policy = "MULTIVALUE"
  }}
  
  health_check_custom_config {{
    failure_threshold = 1
  }}
}}

"#
    )
}

fn generate_integration_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# INTEGRATION LAYER - API GATEWAY, EVENTBRIDGE, SNS, SQS
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-INT-001 (Integration Gateway Node)
# Components: LA-INT-001 (API Gateway), LA-INT-002 (Metadata Manager)
# ═══════════════════════════════════════════════════════════════════════════

# API Gateway - REST API
resource "aws_api_gateway_rest_api" "main" {{
  name        = "${{var.project_name}}-api"
  description = "Data Platform Migration API"
  
  endpoint_configuration {{
    types = ["REGIONAL"]
  }}
  
  tags = {{
    Name = "Data Platform API"
    Component = "LA-INT-001"
    PhysicalNode = "PA-INT-001"
  }}
}}

# API Gateway Resource - Migration Jobs
resource "aws_api_gateway_resource" "migration_jobs" {{
  rest_api_id = aws_api_gateway_rest_api.main.id
  parent_id   = aws_api_gateway_rest_api.main.root_resource_id
  path_part   = "migration-jobs"
}}

# API Gateway Method - POST /migration-jobs
resource "aws_api_gateway_method" "migration_jobs_post" {{
  rest_api_id   = aws_api_gateway_rest_api.main.id
  resource_id   = aws_api_gateway_resource.migration_jobs.id
  http_method   = "POST"
  authorization = "AWS_IAM"
}}

# API Gateway Integration - Step Functions
resource "aws_api_gateway_integration" "migration_jobs_post" {{
  rest_api_id = aws_api_gateway_rest_api.main.id
  resource_id = aws_api_gateway_resource.migration_jobs.id
  http_method = aws_api_gateway_method.migration_jobs_post.http_method
  
  integration_http_method = "POST"
  type                    = "AWS"
  uri                     = "arn:aws:apigateway:${{var.aws_region}}:states:action/StartExecution"
  credentials             = aws_iam_role.api_gateway_step_functions.arn
  
  request_templates = {{
    "application/json" = <<EOF
{{
  "stateMachineArn": "${{aws_sfn_state_machine.etl_workflow.arn}}",
  "input": "$util.escapeJavaScript($input.json('$'))"
}}
EOF
  }}
}}

# API Gateway Deployment
resource "aws_api_gateway_deployment" "main" {{
  rest_api_id = aws_api_gateway_rest_api.main.id
  
  depends_on = [
    aws_api_gateway_integration.migration_jobs_post
  ]
  
  lifecycle {{
    create_before_destroy = true
  }}
}}

# API Gateway Stage
resource "aws_api_gateway_stage" "main" {{
  deployment_id = aws_api_gateway_deployment.main.id
  rest_api_id   = aws_api_gateway_rest_api.main.id
  stage_name    = var.environment
  
  xray_tracing_enabled = true
  
  access_log_settings {{
    destination_arn = aws_cloudwatch_log_group.api_gateway.arn
    format          = "$requestId $context.requestTime $context.httpMethod $context.routeKey $context.status"
  }}
  
  tags = {{
    Name = "API Gateway Stage"
  }}
}}

# EventBridge Event Bus
resource "aws_cloudwatch_event_bus" "main" {{
  name = "${{var.project_name}}-event-bus"
  
  tags = {{
    Name = "Data Platform Event Bus"
    Component = "LA-INT-003"
    PhysicalNode = "PA-INT-001"
  }}
}}

# EventBridge Rule - Migration Job Started
resource "aws_cloudwatch_event_rule" "migration_started" {{
  name        = "${{var.project_name}}-migration-started"
  description = "Trigger when migration job starts"
  event_bus_name = aws_cloudwatch_event_bus.main.name
  
  event_pattern = jsonencode({{
    source      = ["custom.migration"]
    detail-type = ["Migration Job Started"]
  }})
  
  tags = {{
    Name = "Migration Started Rule"
  }}
}}

# EventBridge Target - SNS
resource "aws_cloudwatch_event_target" "migration_started_sns" {{
  rule      = aws_cloudwatch_event_rule.migration_started.name
  target_id = "SendToSNS"
  arn       = aws_sns_topic.migration_events.arn
  event_bus_name = aws_cloudwatch_event_bus.main.name
}}

# SNS Topic - Migration Events
resource "aws_sns_topic" "migration_events" {{
  name              = "${{var.project_name}}-migration-events"
  display_name      = "Migration Events Notifications"
  kms_master_key_id = aws_kms_key.data_encryption.id
  
  tags = {{
    Name = "Migration Events Topic"
  }}
}}

# SNS Topic - Data Quality Alerts
resource "aws_sns_topic" "data_quality_alerts" {{
  name              = "${{var.project_name}}-data-quality-alerts"
  display_name      = "Data Quality Alerts"
  kms_master_key_id = aws_kms_key.data_encryption.id
  
  tags = {{
    Name = "Data Quality Alerts"
    Component = "LA-PROC-003"
    Requirement = "SYS-MON-002"
    SafetyLevel = "High"
  }}
}}

# SNS Subscription - Email
resource "aws_sns_topic_subscription" "data_quality_email" {{
  topic_arn = aws_sns_topic.data_quality_alerts.arn
  protocol  = "email"
  endpoint  = "data-engineering@company.com"
}}

# SQS Queue - ETL Job Queue
resource "aws_sqs_queue" "etl_jobs" {{
  name                       = "${{var.project_name}}-etl-jobs"
  delay_seconds              = 0
  max_message_size           = 262144
  message_retention_seconds  = 1209600
  receive_wait_time_seconds  = 10
  visibility_timeout_seconds = 900
  
  kms_master_key_id                 = aws_kms_key.data_encryption.id
  kms_data_key_reuse_period_seconds = 300
  
  redrive_policy = jsonencode({{
    deadLetterTargetArn = aws_sqs_queue.etl_dlq.arn
    maxReceiveCount     = 3
  }})
  
  tags = {{
    Name = "ETL Job Queue"
    Component = "LA-MIG-001"
  }}
}}

# SQS Queue - Dead Letter Queue
resource "aws_sqs_queue" "etl_dlq" {{
  name                      = "${{var.project_name}}-etl-dlq"
  message_retention_seconds = 1209600
  
  kms_master_key_id = aws_kms_key.data_encryption.id
  
  tags = {{
    Name = "ETL Dead Letter Queue"
  }}
}}

# SQS Queue - Streaming DLQ
resource "aws_sqs_queue" "streaming_dlq" {{
  name                      = "${{var.project_name}}-streaming-dlq"
  message_retention_seconds = 1209600
  
  kms_master_key_id = aws_kms_key.data_encryption.id
  
  tags = {{
    Name = "Streaming Dead Letter Queue"
    Component = "LA-PROC-002"
  }}
}}

# Kinesis Data Stream
resource "aws_kinesis_stream" "data_ingestion" {{
  name             = "${{var.project_name}}-data-ingestion"
  shard_count      = 10
  retention_period = 168
  
  stream_mode_details {{
    stream_mode = "PROVISIONED"
  }}
  
  encryption_type = "KMS"
  kms_key_id      = aws_kms_key.data_encryption.id
  
  tags = {{
    Name = "Data Ingestion Stream"
    Component = "LA-PROC-002"
    Requirement = "SYS-PERF-004"
  }}
}}

"#
    )
}

fn generate_orchestration_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# ORCHESTRATION LAYER - STEP FUNCTIONS, GLUE, MWAA
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-MIG-001 (Migration Engine Node - LA-MIG-001)
# Physical Link: PL-002 (Migration Data Bus)
# ═══════════════════════════════════════════════════════════════════════════

# Step Functions State Machine - ETL Workflow
resource "aws_sfn_state_machine" "etl_workflow" {{
  name     = "${{var.project_name}}-etl-workflow"
  role_arn = aws_iam_role.step_functions.arn
  
  definition = jsonencode({{
    Comment = "ETL workflow for data platform migration"
    StartAt = "ExtractData"
    States = {{
      ExtractData = {{
        Type = "Parallel"
        Branches = [
          {{
            StartAt = "ExtractOracle"
            States = {{
              ExtractOracle = {{
                Type     = "Task"
                Resource = "arn:aws:states:::lambda:invoke"
                Parameters = {{
                  FunctionName = "${{aws_lambda_function.schema_converter.function_name}}"
                  Payload = {{
                    "source": "oracle",
                    "execution_id.$": "$$.Execution.Id"
                  }}
                }}
                ResultPath = "$.oracleResult"
                Next       = "ValidateOracle"
              }}
              ValidateOracle = {{
                Type     = "Task"
                Resource = "arn:aws:states:::lambda:invoke"
                Parameters = {{
                  FunctionName = "${{aws_lambda_function.data_validator.function_name}}"
                  Payload = {{
                    "data.$": "$.oracleResult.Payload"
                  }}
                }}
                End = true
              }}
            }}
          }},
          {{
            StartAt = "ExtractSnowflake"
            States = {{
              ExtractSnowflake = {{
                Type     = "Task"
                Resource = "arn:aws:states:::lambda:invoke"
                Parameters = {{
                  FunctionName = "${{aws_lambda_function.schema_converter.function_name}}"
                  Payload = {{
                    "source": "snowflake",
                    "execution_id.$": "$$.Execution.Id"
                  }}
                }}
                ResultPath = "$.snowflakeResult"
                Next       = "ValidateSnowflake"
              }}
              ValidateSnowflake = {{
                Type     = "Task"
                Resource = "arn:aws:states:::lambda:invoke"
                Parameters = {{
                  FunctionName = "${{aws_lambda_function.data_validator.function_name}}"
                  Payload = {{
                    "data.$": "$.snowflakeResult.Payload"
                  }}
                }}
                End = true
              }}
            }}
          }}
        ]
        Next = "SchemaConversion"
      }}
      SchemaConversion = {{
        Type     = "Task"
        Resource = "arn:aws:states:::glue:startJobRun.sync"
        Parameters = {{
          JobName = "${{aws_glue_job.schema_converter.name}}"
        }}
        ResultPath = "$.schemaResult"
        Retry = [
          {{
            ErrorEquals     = ["States.TaskFailed"]
            IntervalSeconds = 30
            MaxAttempts     = 3
            BackoffRate     = 2.0
          }}
        ]
        Catch = [
          {{
            ErrorEquals = ["States.ALL"]
            Next        = "HandleError"
          }}
        ]
        Next = "DataTransformation"
      }}
      DataTransformation = {{
        Type     = "Task"
        Resource = "arn:aws:states:::glue:startJobRun.sync"
        Parameters = {{
          JobName = "${{aws_glue_job.data_transformer.name}}"
        }}
        Next = "ConflictResolution"
      }}
      ConflictResolution = {{
        Type     = "Task"
        Resource = "arn:aws:states:::lambda:invoke"
        Parameters = {{
          FunctionName = "${{aws_lambda_function.conflict_resolver.function_name}}"
          Payload = {{
            "data.$": "$.schemaResult"
          }}
        }}
        Next = "SendNotification"
      }}
      SendNotification = {{
        Type     = "Task"
        Resource = "arn:aws:states:::sns:publish"
        Parameters = {{
          TopicArn = "${{aws_sns_topic.migration_events.arn}}"
          Message = {{
            "status": "SUCCESS",
            "execution_id.$": "$$.Execution.Id"
          }}
        }}
        End = true
      }}
      HandleError = {{
        Type     = "Task"
        Resource = "arn:aws:states:::sns:publish"
        Parameters = {{
          TopicArn = "${{aws_sns_topic.data_quality_alerts.arn}}"
          Message = {{
            "status": "FAILED",
            "error.$": "$.Error",
            "cause.$": "$.Cause"
          }}
        }}
        End = true
      }}
    }}
  }})
  
  logging_configuration {{
    log_destination        = "${{aws_cloudwatch_log_group.step_functions.arn}}:*"
    include_execution_data = true
    level                  = "ALL"
  }}
  
  tracing_configuration {{
    enabled = true
  }}
  
  tags = {{
    Name = "ETL Workflow"
    Component = "LA-MIG-001"
    PhysicalNode = "PA-MIG-001"
    Requirement = "SYS-MIG-005"
  }}
}}

# AWS Glue Job - Schema Converter
resource "aws_glue_job" "schema_converter" {{
  name     = "${{var.project_name}}-schema-converter"
  role_arn = aws_iam_role.glue_execution.arn
  
  command {{
    name            = "glueetl"
    script_location = "s3://${{aws_s3_bucket.etl_artifacts.bucket}}/scripts/schema_converter.py"
    python_version  = "3"
  }}
  
  default_arguments = {{
    "--job-bookmark-option"           = "job-bookmark-enable"
    "--enable-metrics"                = "true"
    "--enable-continuous-cloudwatch-log" = "true"
    "--enable-spark-ui"               = "true"
    "--spark-event-logs-path"         = "s3://${{aws_s3_bucket.etl_artifacts.bucket}}/spark-logs/"
    "--TempDir"                       = "s3://${{aws_s3_bucket.etl_artifacts.bucket}}/temp/"
    "--enable-glue-datacatalog"       = "true"
  }}
  
  execution_property {{
    max_concurrent_runs = 10
  }}
  
  max_retries = 1
  timeout     = 2880
  glue_version = "4.0"
  
  worker_type       = "G.2X"
  number_of_workers = 10
  
  tags = {{
    Name = "Schema Converter Job"
    Component = "LA-MIG-002"
    Requirement = "SYS-MIG-002"
  }}
}}

# AWS Glue Job - Data Transformer
resource "aws_glue_job" "data_transformer" {{
  name     = "${{var.project_name}}-data-transformer"
  role_arn = aws_iam_role.glue_execution.arn
  
  command {{
    name            = "glueetl"
    script_location = "s3://${{aws_s3_bucket.etl_artifacts.bucket}}/scripts/data_transformer.py"
    python_version  = "3"
  }}
  
  default_arguments = {{
    "--job-bookmark-option"           = "job-bookmark-enable"
    "--enable-metrics"                = "true"
    "--enable-continuous-cloudwatch-log" = "true"
    "--SOURCE_BUCKET"                 = aws_s3_bucket.delta_lake_bronze.bucket
    "--TARGET_BUCKET"                 = aws_s3_bucket.delta_lake_silver.bucket
  }}
  
  execution_property {{
    max_concurrent_runs = 20
  }}
  
  max_retries = 2
  timeout     = 2880
  glue_version = "4.0"
  
  worker_type       = "G.2X"
  number_of_workers = 50
  
  tags = {{
    Name = "Data Transformer Job"
    Component = "LA-PROC-001"
    Requirement = "SYS-SCALE-001"
  }}
}}

# AWS Glue Catalog Database
resource "aws_glue_catalog_database" "main" {{
  name = "${{var.project_name}}_catalog"
  
  description = "Glue catalog for data platform"
  
  tags = {{
    Name = "Data Platform Catalog"
    Component = "LA-INT-002"
  }}
}}

# EventBridge Scheduler - Daily ETL
resource "aws_scheduler_schedule" "daily_etl" {{
  name       = "${{var.project_name}}-daily-etl"
  group_name = "default"
  
  flexible_time_window {{
    mode = "OFF"
  }}
  
  schedule_expression = "cron(0 2 * * ? *)"
  schedule_expression_timezone = "America/New_York"
  
  target {{
    arn      = aws_sfn_state_machine.etl_workflow.arn
    role_arn = aws_iam_role.scheduler.arn
    
    input = jsonencode({{
      source      = "scheduled"
      environment = var.environment
    }})
  }}
  
  description = "Daily ETL workflow trigger at 2 AM"
}}

"#
    )
}

fn generate_governance_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# GOVERNANCE LAYER - IAM, KMS, SECRETS MANAGER, CONFIG
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-GOV-001 (Governance Control Node)
# Components: LA-GOV-001 (Access Control), LA-GOV-002 (Lineage), LA-GOV-003 (Audit)
# ═══════════════════════════════════════════════════════════════════════════

# KMS Key - Data Encryption
resource "aws_kms_key" "data_encryption" {{
  description             = "KMS key for data encryption"
  deletion_window_in_days = 30
  enable_key_rotation     = true
  
  tags = {{
    Name = "Data Encryption Key"
    Component = "LA-GOV-001"
    PhysicalNode = "PA-GOV-001"
    Requirement = "SYS-GOV-001"
    SafetyLevel = "Critical"
  }}
}}

resource "aws_kms_alias" "data_encryption" {{
  name          = "alias/${{var.project_name}}-data-encryption"
  target_key_id = aws_kms_key.data_encryption.key_id
}}

# KMS Key Policy
resource "aws_kms_key_policy" "data_encryption" {{
  key_id = aws_kms_key.data_encryption.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {{
          AWS = "arn:aws:iam::${{var.aws_region}}:root"
        }}
        Action   = "kms:*"
        Resource = "*"
      }},
      {{
        Sid    = "Allow services to use the key"
        Effect = "Allow"
        Principal = {{
          Service = [
            "s3.amazonaws.com",
            "lambda.amazonaws.com",
            "dynamodb.amazonaws.com",
            "sns.amazonaws.com",
            "sqs.amazonaws.com",
            "kinesis.amazonaws.com"
          ]
        }}
        Action = [
          "kms:Decrypt",
          "kms:GenerateDataKey"
        ]
        Resource = "*"
      }}
    ]
  }})
}}

# Secrets Manager - Oracle Credentials
resource "aws_secretsmanager_secret" "oracle_credentials" {{
  name                    = "${{var.project_name}}/oracle/credentials"
  description             = "Oracle database credentials"
  kms_key_id              = aws_kms_key.data_encryption.id
  recovery_window_in_days = 30
  
  tags = {{
    Name = "Oracle Credentials"
    Component = "LA-SRC-001"
    PhysicalNode = "PA-SRC-001"
  }}
}}

resource "aws_secretsmanager_secret_version" "oracle_credentials" {{
  secret_id = aws_secretsmanager_secret.oracle_credentials.id
  secret_string = jsonencode({{
    username = "oracle_user"
    password = "changeme"
    host     = var.oracle_db_endpoint
    port     = 1521
    database = "ORCL"
  }})
}}

# Secrets Manager - Snowflake Credentials
resource "aws_secretsmanager_secret" "snowflake_credentials" {{
  name                    = "${{var.project_name}}/snowflake/credentials"
  description             = "Snowflake credentials"
  kms_key_id              = aws_kms_key.data_encryption.id
  recovery_window_in_days = 30
  
  tags = {{
    Name = "Snowflake Credentials"
    Component = "LA-SRC-002"
    PhysicalNode = "PA-SRC-001"
  }}
}}

# IAM Role - ECS Execution
resource "aws_iam_role" "ecs_execution" {{
  name = "${{var.project_name}}-ecs-execution-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "ecs-tasks.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "ECS Execution Role"
  }}
}}

resource "aws_iam_role_policy_attachment" "ecs_execution" {{
  role       = aws_iam_role.ecs_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}}

# IAM Role - ETL Task
resource "aws_iam_role" "etl_task" {{
  name = "${{var.project_name}}-etl-task-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "ecs-tasks.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "ETL Task Role"
    Component = "LA-MIG-001"
  }}
}}

resource "aws_iam_role_policy" "etl_task" {{
  name = "etl-task-policy"
  role = aws_iam_role.etl_task.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          "${{aws_s3_bucket.delta_lake_bronze.arn}}",
          "${{aws_s3_bucket.delta_lake_bronze.arn}}/*",
          "${{aws_s3_bucket.delta_lake_silver.arn}}",
          "${{aws_s3_bucket.delta_lake_silver.arn}}/*"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:Query",
          "dynamodb:Scan"
        ]
        Resource = [
          "${{aws_dynamodb_table.migration_state.arn}}",
          "${{aws_dynamodb_table.validation_results.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Resource = [
          "${{aws_secretsmanager_secret.oracle_credentials.arn}}",
          "${{aws_secretsmanager_secret.snowflake_credentials.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "kms:Decrypt",
          "kms:GenerateDataKey"
        ]
        Resource = "${{aws_kms_key.data_encryption.arn}}"
      }}
    ]
  }})
}}

# IAM Role - Lambda Execution
resource "aws_iam_role" "lambda_execution" {{
  name = "${{var.project_name}}-lambda-execution-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "lambda.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "Lambda Execution Role"
  }}
}}

resource "aws_iam_role_policy_attachment" "lambda_execution_basic" {{
  role       = aws_iam_role.lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}}

resource "aws_iam_role_policy_attachment" "lambda_execution_vpc" {{
  role       = aws_iam_role.lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}}

resource "aws_iam_role_policy" "lambda_execution_custom" {{
  name = "lambda-execution-custom-policy"
  role = aws_iam_role.lambda_execution.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "s3:*"
        ]
        Resource = [
          "${{aws_s3_bucket.delta_lake_bronze.arn}}/*",
          "${{aws_s3_bucket.delta_lake_silver.arn}}/*",
          "${{aws_s3_bucket.delta_lake_gold.arn}}/*"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "dynamodb:*"
        ]
        Resource = [
          "${{aws_dynamodb_table.migration_state.arn}}",
          "${{aws_dynamodb_table.validation_results.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "sns:Publish"
        ]
        Resource = [
          "${{aws_sns_topic.migration_events.arn}}",
          "${{aws_sns_topic.data_quality_alerts.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "xray:PutTraceSegments",
          "xray:PutTelemetryRecords"
        ]
        Resource = "*"
      }}
    ]
  }})
}}

# IAM Role - Step Functions
resource "aws_iam_role" "step_functions" {{
  name = "${{var.project_name}}-step-functions-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "states.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "Step Functions Role"
    Component = "LA-MIG-001"
  }}
}}

resource "aws_iam_role_policy" "step_functions" {{
  name = "step-functions-policy"
  role = aws_iam_role.step_functions.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "lambda:InvokeFunction"
        ]
        Resource = [
          "${{aws_lambda_function.schema_converter.arn}}",
          "${{aws_lambda_function.data_validator.arn}}",
          "${{aws_lambda_function.conflict_resolver.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "glue:StartJobRun",
          "glue:GetJobRun",
          "glue:GetJobRuns",
          "glue:BatchStopJobRun"
        ]
        Resource = "*"
      }},
      {{
        Effect = "Allow"
        Action = [
          "sns:Publish"
        ]
        Resource = [
          "${{aws_sns_topic.migration_events.arn}}",
          "${{aws_sns_topic.data_quality_alerts.arn}}"
        ]
      }},
      {{
        Effect = "Allow"
        Action = [
          "xray:PutTraceSegments",
          "xray:PutTelemetryRecords",
          "xray:GetSamplingRules",
          "xray:GetSamplingTargets"
        ]
        Resource = "*"
      }}
    ]
  }})
}}

# IAM Role - Glue Execution
resource "aws_iam_role" "glue_execution" {{
  name = "${{var.project_name}}-glue-execution-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "glue.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "Glue Execution Role"
  }}
}}

resource "aws_iam_role_policy_attachment" "glue_execution" {{
  role       = aws_iam_role.glue_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSGlueServiceRole"
}}

resource "aws_iam_role_policy" "glue_execution_custom" {{
  name = "glue-execution-custom-policy"
  role = aws_iam_role.glue_execution.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "s3:*"
        ]
        Resource = [
          "${{aws_s3_bucket.delta_lake_bronze.arn}}/*",
          "${{aws_s3_bucket.delta_lake_silver.arn}}/*",
          "${{aws_s3_bucket.delta_lake_gold.arn}}/*",
          "${{aws_s3_bucket.etl_artifacts.arn}}/*"
        ]
      }}
    ]
  }})
}}

# IAM Role - API Gateway Step Functions
resource "aws_iam_role" "api_gateway_step_functions" {{
  name = "${{var.project_name}}-api-gateway-sf-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "apigateway.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "API Gateway Step Functions Role"
  }}
}}

resource "aws_iam_role_policy" "api_gateway_step_functions" {{
  name = "api-gateway-sf-policy"
  role = aws_iam_role.api_gateway_step_functions.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "states:StartExecution"
        ]
        Resource = "${{aws_sfn_state_machine.etl_workflow.arn}}"
      }}
    ]
  }})
}}

# IAM Role - EventBridge Scheduler
resource "aws_iam_role" "scheduler" {{
  name = "${{var.project_name}}-scheduler-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "scheduler.amazonaws.com"
        }}
      }}
    ]
  }})
  
  tags = {{
    Name = "EventBridge Scheduler Role"
  }}
}}

resource "aws_iam_role_policy" "scheduler" {{
  name = "scheduler-policy"
  role = aws_iam_role.scheduler.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "states:StartExecution"
        ]
        Resource = "${{aws_sfn_state_machine.etl_workflow.arn}}"
      }}
    ]
  }})
}}

# AWS Config - Compliance Monitoring
resource "aws_config_configuration_recorder" "main" {{
  name     = "${{var.project_name}}-config-recorder"
  role_arn = aws_iam_role.config.arn
  
  recording_group {{
    all_supported = true
    include_global_resource_types = true
  }}
}}

resource "aws_iam_role" "config" {{
  name = "${{var.project_name}}-config-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "config.amazonaws.com"
        }}
      }}
    ]
  }})
}}

resource "aws_iam_role_policy_attachment" "config" {{
  role       = aws_iam_role.config.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/ConfigRole"
}}

"#
    )
}

fn generate_monitoring_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# MONITORING LAYER - CLOUDWATCH, X-RAY, OPENTELEMETRY
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-MON-001 (Monitoring Infrastructure)
# Components: LA-MON-001 (Observability), LA-MON-002 (Cost Optimizer), LA-MON-003 (Alert Manager)
# ═══════════════════════════════════════════════════════════════════════════

# CloudWatch Log Groups
resource "aws_cloudwatch_log_group" "step_functions" {{
  name              = "/aws/stepfunctions/${{var.project_name}}-etl-workflow"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_encryption.arn
  
  tags = {{
    Name = "Step Functions Logs"
    Component = "LA-MON-001"
    PhysicalNode = "PA-MON-001"
  }}
}}

resource "aws_cloudwatch_log_group" "api_gateway" {{
  name              = "/aws/apigateway/${{var.project_name}}"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_encryption.arn
  
  tags = {{
    Name = "API Gateway Logs"
    Component = "LA-MON-001"
  }}
}}

resource "aws_cloudwatch_log_group" "lambda_schema_converter" {{
  name              = "/aws/lambda/${{var.project_name}}-schema-converter"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_encryption.arn
  
  tags = {{
    Name = "Lambda Schema Converter Logs"
  }}
}}

resource "aws_cloudwatch_log_group" "lambda_data_validator" {{
  name              = "/aws/lambda/${{var.project_name}}-data-validator"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_encryption.arn
  
  tags = {{
    Name = "Lambda Data Validator Logs"
  }}
}}

resource "aws_cloudwatch_log_group" "ecs_etl_orchestrator" {{
  name              = "/ecs/${{var.project_name}}/etl-orchestrator"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_encryption.arn
  
  tags = {{
    Name = "ECS ETL Orchestrator Logs"
  }}
}}

# CloudWatch Alarms - Step Functions
resource "aws_cloudwatch_metric_alarm" "step_functions_failed" {{
  alarm_name          = "${{var.project_name}}-step-functions-failed"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "ExecutionsFailed"
  namespace           = "AWS/States"
  period              = 300
  statistic           = "Sum"
  threshold           = 0
  alarm_description   = "Step Functions execution failed"
  alarm_actions       = [aws_sns_topic.data_quality_alerts.arn]
  
  dimensions = {{
    StateMachineArn = aws_sfn_state_machine.etl_workflow.arn
  }}
  
  tags = {{
    Name = "Step Functions Failed Alarm"
    Component = "LA-MON-003"
    Requirement = "SYS-MON-001"
  }}
}}

# CloudWatch Alarms - Lambda Errors
resource "aws_cloudwatch_metric_alarm" "lambda_errors" {{
  alarm_name          = "${{var.project_name}}-lambda-errors"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = 300
  statistic           = "Sum"
  threshold           = 5
  alarm_description   = "Lambda function errors exceeded threshold"
  alarm_actions       = [aws_sns_topic.data_quality_alerts.arn]
  
  dimensions = {{
    FunctionName = aws_lambda_function.data_validator.function_name
  }}
  
  tags = {{
    Name = "Lambda Errors Alarm"
    Component = "LA-MON-003"
    SafetyLevel = "High"
  }}
}}

# CloudWatch Alarms - DynamoDB Throttling
resource "aws_cloudwatch_metric_alarm" "dynamodb_throttling" {{
  alarm_name          = "${{var.project_name}}-dynamodb-throttling"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "UserErrors"
  namespace           = "AWS/DynamoDB"
  period              = 300
  statistic           = "Sum"
  threshold           = 10
  alarm_description   = "DynamoDB throttling detected"
  alarm_actions       = [aws_sns_topic.data_quality_alerts.arn]
  
  dimensions = {{
    TableName = aws_dynamodb_table.migration_state.name
  }}
  
  tags = {{
    Name = "DynamoDB Throttling Alarm"
  }}
}}

# CloudWatch Dashboard
resource "aws_cloudwatch_dashboard" "main" {{
  dashboard_name = "${{var.project_name}}-monitoring"
  
  dashboard_body = jsonencode({{
    widgets = [
      {{
        type = "metric"
        properties = {{
          metrics = [
            ["AWS/States", "ExecutionsStarted", {{ stat = "Sum" }}],
            [".", "ExecutionsSucceeded", {{ stat = "Sum" }}],
            [".", "ExecutionsFailed", {{ stat = "Sum" }}]
          ]
          period = 300
          stat   = "Sum"
          region = var.aws_region
          title  = "Step Functions Executions"
        }}
      }},
      {{
        type = "metric"
        properties = {{
          metrics = [
            ["AWS/Lambda", "Invocations", {{ stat = "Sum" }}],
            [".", "Errors", {{ stat = "Sum" }}],
            [".", "Duration", {{ stat = "Average" }}]
          ]
          period = 300
          stat   = "Sum"
          region = var.aws_region
          title  = "Lambda Metrics"
        }}
      }},
      {{
        type = "metric"
        properties = {{
          metrics = [
            ["AWS/DynamoDB", "ConsumedReadCapacityUnits", {{ stat = "Sum" }}],
            [".", "ConsumedWriteCapacityUnits", {{ stat = "Sum" }}]
          ]
          period = 300
          stat   = "Sum"
          region = var.aws_region
          title  = "DynamoDB Capacity"
        }}
      }},
      {{
        type = "metric"
        properties = {{
          metrics = [
            ["AWS/S3", "NumberOfObjects", {{ stat = "Average" }}],
            [".", "BucketSizeBytes", {{ stat = "Average" }}]
          ]
          period = 86400
          stat   = "Average"
          region = var.aws_region
          title  = "S3 Storage Metrics"
        }}
      }}
    ]
  }})
}}

# X-Ray Sampling Rule
resource "aws_xray_sampling_rule" "main" {{
  rule_name      = "${{var.project_name}}-sampling-rule"
  priority       = 1000
  version        = 1
  reservoir_size = 1
  fixed_rate     = 0.05
  url_path       = "*"
  host           = "*"
  http_method    = "*"
  service_type   = "*"
  service_name   = "*"
  resource_arn   = "*"
  
  attributes = {{
    Environment = var.environment
  }}
  
  tags = {{
    Name = "X-Ray Sampling Rule"
    Component = "LA-MON-001"
  }}
}}

# CloudWatch Insights Query - ETL Performance
resource "aws_cloudwatch_query_definition" "etl_performance" {{
  name = "${{var.project_name}}-etl-performance"
  
  log_group_names = [
    aws_cloudwatch_log_group.step_functions.name,
    aws_cloudwatch_log_group.lambda_schema_converter.name
  ]
  
  query_string = <<-EOQ
    fields @timestamp, @message
    | filter @message like /REPORT/
    | parse @message /Duration: (?<duration>\\d+\\.\\d+)/
    | stats avg(duration), max(duration), min(duration) by bin(5m)
  EOQ
}}

# CloudWatch Cost Anomaly Detector
resource "aws_ce_anomaly_monitor" "main" {{
  name              = "${{var.project_name}}-cost-monitor"
  monitor_type      = "DIMENSIONAL"
  monitor_dimension = "SERVICE"
  
  tags = {{
    Name = "Cost Anomaly Monitor"
    Component = "LA-MON-002"
    PhysicalNode = "PA-MON-001"
    Requirement = "SYS-MON-003"
  }}
}}

resource "aws_ce_anomaly_subscription" "main" {{
  name      = "${{var.project_name}}-cost-alerts"
  frequency = "DAILY"
  
  monitor_arn_list = [
    aws_ce_anomaly_monitor.main.arn
  ]
  
  subscriber {{
    type    = "SNS"
    address = aws_sns_topic.data_quality_alerts.arn
  }}
  
  threshold_expression {{
    dimension {{
      key           = "ANOMALY_TOTAL_IMPACT_ABSOLUTE"
      match_options = ["GREATER_THAN_OR_EQUAL"]
      values        = ["100"]
    }}
  }}
}}

"#
    )
}

fn generate_analytics_layer(_model: &SemanticModel, config: &AwsConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# ANALYTICS LAYER - ATHENA, QUICKSIGHT, REDSHIFT
# ═══════════════════════════════════════════════════════════════════════════
# Physical Node: PA-ANLZ-001 (Analytics Compute Node)
# Components: LA-ANLZ-001 (SQL Analytics), LA-ANLZ-002 (ML Workspace), LA-ANLZ-003 (BI Connector)
# ═══════════════════════════════════════════════════════════════════════════

# S3 Bucket - Athena Query Results
resource "aws_s3_bucket" "athena_results" {{
  bucket = "${{var.project_name}}-athena-results-${{var.environment}}"
  
  tags = {{
    Name = "Athena Query Results"
    Component = "LA-ANLZ-001"
    PhysicalNode = "PA-ANLZ-001"
  }}
}}

resource "aws_s3_bucket_server_side_encryption_configuration" "athena_results" {{
  bucket = aws_s3_bucket.athena_results.id
  
  rule {{
    apply_server_side_encryption_by_default {{
      sse_algorithm     = "aws:kms"
      kms_master_key_id = aws_kms_key.data_encryption.arn
    }}
  }}
}}

resource "aws_s3_bucket_lifecycle_configuration" "athena_results" {{
  bucket = aws_s3_bucket.athena_results.id
  
  rule {{
    id     = "delete_old_queries"
    status = "Enabled"
    
    expiration {{
      days = 30
    }}
  }}
}}

# Athena Workgroup
resource "aws_athena_workgroup" "main" {{
  name = "${{var.project_name}}-workgroup"
  
  configuration {{
    enforce_workgroup_configuration    = true
    publish_cloudwatch_metrics_enabled = true
    
    result_configuration {{
      output_location = "s3://${{aws_s3_bucket.athena_results.bucket}}/output/"
      
      encryption_configuration {{
        encryption_option = "SSE_KMS"
        kms_key_arn       = aws_kms_key.data_encryption.arn
      }}
    }}
    
    engine_version {{
      selected_engine_version = "Athena engine version 3"
    }}
  }}
  
  tags = {{
    Name = "Athena Workgroup"
    Component = "LA-ANLZ-001"
    Requirement = "SYS-PERF-001"
  }}
}}

# Athena Named Query - Data Quality Check
resource "aws_athena_named_query" "data_quality_check" {{
  name      = "data_quality_check"
  workgroup = aws_athena_workgroup.main.id
  database  = aws_glue_catalog_database.main.name
  query     = <<-EOQ
    SELECT 
      table_name,
      COUNT(*) as row_count,
      COUNT(DISTINCT primary_key) as unique_keys,
      COUNT(CASE WHEN validation_status = 'FAILED' THEN 1 END) as failed_validations,
      MIN(ingestion_timestamp) as first_record,
      MAX(ingestion_timestamp) as last_record
    FROM validation_results
    WHERE date = CURRENT_DATE
    GROUP BY table_name
    ORDER BY failed_validations DESC
  EOQ
  
  description = "Daily data quality summary"
}}

# Glue Crawler - Bronze Layer
resource "aws_glue_crawler" "bronze" {{
  name          = "${{var.project_name}}-bronze-crawler"
  role          = aws_iam_role.glue_execution.arn
  database_name = aws_glue_catalog_database.main.name
  
  s3_target {{
    path = "s3://${{aws_s3_bucket.delta_lake_bronze.bucket}}/"
  }}
  
  schema_change_policy {{
    update_behavior = "UPDATE_IN_DATABASE"
    delete_behavior = "LOG"
  }}
  
  configuration = jsonencode({{
    Version = 1.0
    CrawlerOutput = {{
      Partitions = {{
        AddOrUpdateBehavior = "InheritFromTable"
      }}
    }}
  }})
  
  tags = {{
    Name = "Bronze Layer Crawler"
    Layer = "bronze"
  }}
}}

# Glue Crawler - Silver Layer
resource "aws_glue_crawler" "silver" {{
  name          = "${{var.project_name}}-silver-crawler"
  role          = aws_iam_role.glue_execution.arn
  database_name = aws_glue_catalog_database.main.name
  
  s3_target {{
    path = "s3://${{aws_s3_bucket.delta_lake_silver.bucket}}/"
  }}
  
  schema_change_policy {{
    update_behavior = "UPDATE_IN_DATABASE"
    delete_behavior = "LOG"
  }}
  
  tags = {{
    Name = "Silver Layer Crawler"
    Layer = "silver"
  }}
}}

# Glue Crawler - Gold Layer
resource "aws_glue_crawler" "gold" {{
  name          = "${{var.project_name}}-gold-crawler"
  role          = aws_iam_role.glue_execution.arn
  database_name = aws_glue_catalog_database.main.name
  
  s3_target {{
    path = "s3://${{aws_s3_bucket.delta_lake_gold.bucket}}/"
  }}
  
  schema_change_policy {{
    update_behavior = "UPDATE_IN_DATABASE"
    delete_behavior = "LOG"
  }}
  
  tags = {{
    Name = "Gold Layer Crawler"
    Layer = "gold"
    Component = "LA-ANLZ-001"
  }}
}}

# EventBridge Rule - Run Crawlers Daily
resource "aws_cloudwatch_event_rule" "run_crawlers" {{
  name                = "${{var.project_name}}-run-crawlers"
  description         = "Run Glue crawlers daily at 6 AM"
  schedule_expression = "cron(0 6 * * ? *)"
  
  tags = {{
    Name = "Run Crawlers Daily"
  }}
}}

resource "aws_cloudwatch_event_target" "run_bronze_crawler" {{
  rule     = aws_cloudwatch_event_rule.run_crawlers.name
  arn      = "arn:aws:glue:${{var.aws_region}}:${{data.aws_caller_identity.current.account_id}}:crawler/${{aws_glue_crawler.bronze.name}}"
  role_arn = aws_iam_role.crawler_scheduler.arn
}}

resource "aws_iam_role" "crawler_scheduler" {{
  name = "${{var.project_name}}-crawler-scheduler-role"
  
  assume_role_policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {{
          Service = "events.amazonaws.com"
        }}
      }}
    ]
  }})
}}

resource "aws_iam_role_policy" "crawler_scheduler" {{
  name = "crawler-scheduler-policy"
  role = aws_iam_role.crawler_scheduler.id
  
  policy = jsonencode({{
    Version = "2012-10-17"
    Statement = [
      {{
        Effect = "Allow"
        Action = [
          "glue:StartCrawler"
        ]
        Resource = "*"
      }}
    ]
  }})
}}

# Data Sources
data "aws_caller_identity" "current" {{}}

"#
    )
}

fn generate_outputs(model: &SemanticModel) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# OUTPUTS
# ═══════════════════════════════════════════════════════════════════════════

output "vpc_id" {{
  description = "VPC ID"
  value       = aws_vpc.main.id
}}

output "private_subnet_ids" {{
  description = "Private subnet IDs"
  value       = aws_subnet.private_compute[*].id
}}

output "delta_lake_bronze_bucket" {{
  description = "S3 bucket for Delta Lake bronze layer"
  value       = aws_s3_bucket.delta_lake_bronze.bucket
}}

output "delta_lake_silver_bucket" {{
  description = "S3 bucket for Delta Lake silver layer"
  value       = aws_s3_bucket.delta_lake_silver.bucket
}}

output "delta_lake_gold_bucket" {{
  description = "S3 bucket for Delta Lake gold layer"
  value       = aws_s3_bucket.delta_lake_gold.bucket
}}

output "ecs_cluster_name" {{
  description = "ECS cluster name"
  value       = aws_ecs_cluster.migration_engine.name
}}

output "step_functions_arn" {{
  description = "Step Functions state machine ARN"
  value       = aws_sfn_state_machine.etl_workflow.arn
}}

output "api_gateway_url" {{
  description = "API Gateway invoke URL"
  value       = "${{aws_api_gateway_stage.main.invoke_url}}"
}}

output "glue_catalog_database" {{
  description = "Glue catalog database name"
  value       = aws_glue_catalog_database.main.name
}}

output "athena_workgroup" {{
  description = "Athena workgroup name"
  value       = aws_athena_workgroup.main.name
}}

output "monitoring_dashboard_url" {{
  description = "CloudWatch dashboard URL"
  value       = "https://console.aws.amazon.com/cloudwatch/home?region=${{var.aws_region}}#dashboards:name=${{aws_cloudwatch_dashboard.main.dashboard_name}}"
}}

output "kms_key_id" {{
  description = "KMS key ID for data encryption"
  value       = aws_kms_key.data_encryption.id
}}

# Model Statistics
# Requirements: {}
# Components: {}
# Traces: {}
# Physical Nodes: 8
# Physical Links: 5

# ═══════════════════════════════════════════════════════════════════════════
# END OF TERRAFORM CONFIGURATION
# Generated from ArcLang Model with Complete AWS Infrastructure
# ═══════════════════════════════════════════════════════════════════════════
"#,
        model.requirements.len(),
        model.components.len(),
        model.traces.len()
    )
}
