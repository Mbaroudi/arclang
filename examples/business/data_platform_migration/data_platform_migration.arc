// ═══════════════════════════════════════════════════════════════════════════
// ENTERPRISE DATA PLATFORM MIGRATION SYSTEM
// ═══════════════════════════════════════════════════════════════════════════
// Domain: Media & Video Streaming Data Platform
// Purpose: Migrate data ecosystem from Oracle/Snowflake to Databricks
// Scope: Enterprise-grade governance, scalability, and zero-downtime migration
// Budget: $3M total ($2M infrastructure, $1M services)
// Timeline: 12 months
// Team: 2 architects, 5 data engineers, 2 analysts
// ═══════════════════════════════════════════════════════════════════════════

model DataPlatformMigration {
    metadata {
        version: "1.0.0"
        description: "Enterprise data platform migration from Oracle/Snowflake to Databricks"
        domain: "media_streaming"
        project_phase: "architecture_design"
        budget_total: "3000000"
        timeline_months: "12"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STAKEHOLDER REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

requirements stakeholder {
    req "STK-001" "Total Cost of Ownership Reduction" {
        description: "Business must reduce total cost of ownership for data platform by 40% over 3 years through consolidation and automation"
        priority: Critical
    }
    
    req "STK-002" "Real-Time and Batch Analytics Support" {
        description: "Platform must support both real-time streaming analytics and batch analytics for video streaming metrics"
        priority: Critical
    }
    
    req "STK-003" "High Availability During Migration" {
        description: "System must maintain 99.9% data availability during migration period with no data loss"
        priority: Critical
    }
    
    req "STK-004" "Unified Platform for Data Teams" {
        description: "Data teams need unified platform combining data engineering, data science, and analytics workloads"
        priority: High
    }
    
    req "STK-005" "Zero Downtime Migration" {
        description: "Business requires seamless user experience with zero planned downtime during migration"
        priority: Critical
    }
    
    req "STK-006" "Scalable Architecture for Growth" {
        description: "Organization needs scalable architecture supporting 10x data growth from 100TB to 1PB over 3 years"
        priority: High
    }
    
    req "STK-007" "Full Data Lineage and Audit Trails" {
        description: "Compliance team requires complete data lineage tracking and audit trails for GDPR and CCPA compliance"
        priority: Critical
    }
    
    req "STK-008" "12-Month Migration Timeline" {
        description: "Leadership needs migration completed within 12 months with clear Q1-Q4 milestones"
        priority: Critical
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - DATA MIGRATION
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-MIG-001" "Bidirectional Data Sync" {
        description: "System shall support bidirectional synchronization between Oracle/Snowflake and Databricks with conflict resolution"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-MIG-002" "Automated Schema Mapping" {
        description: "System shall automatically discover and map schemas from Oracle/Snowflake to Databricks Delta Lake format"
        priority: High
    }
    
    req "SYS-MIG-003" "Data Validation Framework" {
        description: "System shall validate data integrity through row count reconciliation and checksum validation after each migration batch"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-MIG-004" "Rollback Procedures" {
        description: "System shall provide automated rollback capability to previous stable state within 15 minutes"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-MIG-005" "Incremental Migration Waves" {
        description: "System shall support incremental migration in waves with table-level granularity and parallel execution"
        priority: High
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - PERFORMANCE
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-PERF-001" "Query Response Time" {
        description: "System shall deliver query response time under 5 seconds for 95th percentile of analytical queries"
        priority: Critical
    }
    
    req "SYS-PERF-002" "Large Dataset Support" {
        description: "System shall efficiently handle datasets exceeding 100TB with columnar storage and optimization"
        priority: High
    }
    
    req "SYS-PERF-003" "Concurrent User Support" {
        description: "System shall support 500+ concurrent users with workload isolation and auto-scaling"
        priority: High
    }
    
    req "SYS-PERF-004" "Streaming Latency" {
        description: "System shall process streaming data with end-to-end latency under 60 seconds for 99% of events"
        priority: Critical
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - SCALABILITY & GOVERNANCE
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-SCALE-001" "Auto-Scaling Compute" {
        description: "System shall automatically scale compute clusters from 2 to 200 nodes based on workload demand"
        priority: High
    }
    
    req "SYS-SCALE-002" "Elastic Storage" {
        description: "System shall provide elastic cloud storage scaling from 100TB to 1PB+ with automatic tiering"
        priority: High
    }
    
    req "SYS-GOV-001" "Role-Based Access Control" {
        description: "System shall enforce granular RBAC at table, column, and row level with dynamic data masking for PII"
        priority: Critical
        safety_level: Critical
    }
    
    req "SYS-GOV-002" "PII Detection and Classification" {
        description: "System shall automatically detect and classify PII data using ML-based scanning"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-GOV-003" "Complete Data Lineage" {
        description: "System shall capture and visualize end-to-end data lineage with column-level tracking"
        priority: Critical
        safety_level: High
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - RELIABILITY & MONITORING
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-REL-001" "Automated Backup and Recovery" {
        description: "System shall perform automated incremental backups every 6 hours with point-in-time recovery"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-REL-002" "Disaster Recovery SLA" {
        description: "System shall meet disaster recovery SLA with RPO under 1 hour and RTO under 4 hours"
        priority: Critical
        safety_level: High
    }
    
    req "SYS-MON-001" "Real-Time Pipeline Monitoring" {
        description: "System shall provide real-time monitoring of data pipelines with alerts for failures and SLA breaches"
        priority: High
    }
    
    req "SYS-MON-002" "Data Quality Metrics" {
        description: "System shall track data quality metrics with automated anomaly detection and root cause analysis"
        priority: High
    }
    
    req "SYS-MON-003" "Cost Tracking and Optimization" {
        description: "System shall track cloud costs by workload and team with optimization recommendations"
        priority: Medium
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - SOURCE DATA LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Oracle Database Subsystem" {
        id: "LA-SRC-001"
        description: "Legacy Oracle database containing customer data, transactions, and operational metrics (50TB)"
        safety_level: High
    }
    
    component "Snowflake Warehouse Subsystem" {
        id: "LA-SRC-002"
        description: "Existing Snowflake data warehouse with analytics workloads and aggregated metrics (50TB)"
        safety_level: High
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // MIGRATION ENGINE COMPONENTS
    // ═══════════════════════════════════════════════════════════════════════
    
    component "ETL Orchestrator" {
        id: "LA-MIG-001"
        description: "Orchestrates end-to-end migration workflows with wave-based execution and parallel processing"
        safety_level: High
    }
    
    component "Schema Converter" {
        id: "LA-MIG-002"
        description: "Automatically converts Oracle/Snowflake schemas to Databricks Delta Lake format"
        safety_level: High
    }
    
    component "Data Validator" {
        id: "LA-MIG-003"
        description: "Validates migrated data through row count reconciliation, checksum comparison, and sampling"
        safety_level: Critical
    }
    
    component "Conflict Resolver" {
        id: "LA-MIG-004"
        description: "Detects and resolves data conflicts during bidirectional sync with configurable strategies"
        safety_level: High
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // TARGET PLATFORM LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "Databricks Lakehouse Platform" {
        id: "LA-TGT-001"
        description: "Unified lakehouse platform combining data lake and warehouse capabilities with Delta Lake storage"
        safety_level: High
    }
    
    component "Unity Catalog" {
        id: "LA-TGT-002"
        description: "Unified governance solution for data and AI assets with fine-grained access control"
        safety_level: Critical
    }
    
    component "Delta Lake Storage" {
        id: "LA-TGT-003"
        description: "ACID-compliant storage layer with versioning, time travel, and schema evolution"
        safety_level: High
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // DATA PROCESSING LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "Batch Pipeline Engine" {
        id: "LA-PROC-001"
        description: "Processes large-scale batch workloads with medallion architecture (bronze/silver/gold)"
        safety_level: Medium
    }
    
    component "Streaming Pipeline Engine" {
        id: "LA-PROC-002"
        description: "Processes real-time streaming data with exactly-once semantics and sub-minute latency"
        safety_level: Medium
    }
    
    component "Data Quality Engine" {
        id: "LA-PROC-003"
        description: "Enforces data quality rules, detects anomalies, and quarantines invalid data"
        safety_level: High
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // GOVERNANCE LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "Access Control Manager" {
        id: "LA-GOV-001"
        description: "Manages fine-grained access control with RBAC, ABAC, and dynamic data masking"
        safety_level: Critical
    }
    
    component "Lineage Tracker" {
        id: "LA-GOV-002"
        description: "Captures end-to-end data lineage from source systems through transformations to consumption"
        safety_level: High
    }
    
    component "Audit Logger" {
        id: "LA-GOV-003"
        description: "Centralized audit logging for compliance with GDPR, CCPA, SOC2 requirements"
        safety_level: Critical
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // INTEGRATION LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "API Gateway" {
        id: "LA-INT-001"
        description: "RESTful API gateway for data access, job submission, and metadata queries"
        safety_level: High
    }
    
    component "Metadata Manager" {
        id: "LA-INT-002"
        description: "Centralized metadata management for technical, business, and operational metadata"
        safety_level: Medium
    }
    
    component "Workflow Orchestrator" {
        id: "LA-INT-003"
        description: "Orchestrates complex multi-step workflows across data pipelines and ML training"
        safety_level: Medium
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // ANALYTICS LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "SQL Analytics Engine" {
        id: "LA-ANLZ-001"
        description: "SQL-based analytics engine optimized for BI workloads with result caching"
        safety_level: Low
    }
    
    component "ML Workspace" {
        id: "LA-ANLZ-002"
        description: "Machine learning workspace with notebook environment, MLflow tracking, and model serving"
        safety_level: Low
    }
    
    component "BI Connector Hub" {
        id: "LA-ANLZ-003"
        description: "Connectivity layer for BI tools (Tableau, PowerBI, Looker) with semantic layer"
        safety_level: Low
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // MONITORING & OPERATIONS LAYER
    // ═══════════════════════════════════════════════════════════════════════
    
    component "Observability Platform" {
        id: "LA-MON-001"
        description: "Centralized observability with metrics, logs, and traces for all platform components"
        safety_level: Medium
    }
    
    component "Cost Optimizer" {
        id: "LA-MON-002"
        description: "Cloud cost optimization with workload-based recommendations and budget management"
        safety_level: Low
    }
    
    component "Alert Manager" {
        id: "LA-MON-003"
        description: "Intelligent alerting system with anomaly detection, alert routing, and on-call management"
        safety_level: Medium
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTIONS - MIGRATION DATA FLOW
    // ═══════════════════════════════════════════════════════════════════════
    
    connection "OracleToMigrationEngine" {
        from: "LA-SRC-001"
        to: "LA-MIG-001"
    }
    
    connection "SnowflakeToMigrationEngine" {
        from: "LA-SRC-002"
        to: "LA-MIG-001"
    }
    
    connection "MigrationEngineToSchemaConverter" {
        from: "LA-MIG-001"
        to: "LA-MIG-002"
    }
    
    connection "SchemaConverterToValidator" {
        from: "LA-MIG-002"
        to: "LA-MIG-003"
    }
    
    connection "MigrationEngineToDeltaLake" {
        from: "LA-MIG-001"
        to: "LA-TGT-003"
    }
    
    connection "ValidatorToConflictResolver" {
        from: "LA-MIG-003"
        to: "LA-MIG-004"
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTIONS - DATA PROCESSING FLOW
    // ═══════════════════════════════════════════════════════════════════════
    
    connection "DeltaLakeToBatchPipeline" {
        from: "LA-TGT-003"
        to: "LA-PROC-001"
    }
    
    connection "BatchPipelineToDataQuality" {
        from: "LA-PROC-001"
        to: "LA-PROC-003"
    }
    
    connection "StreamingPipelineToDeltaLake" {
        from: "LA-PROC-002"
        to: "LA-TGT-003"
    }
    
    connection "DataQualityToQuarantine" {
        from: "LA-PROC-003"
        to: "LA-TGT-003"
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTIONS - GOVERNANCE & ANALYTICS FLOW
    // ═══════════════════════════════════════════════════════════════════════
    
    connection "UnityCatalogToAccessControl" {
        from: "LA-TGT-002"
        to: "LA-GOV-001"
    }
    
    connection "UnityCatalogToLineageTracker" {
        from: "LA-TGT-002"
        to: "LA-GOV-002"
    }
    
    connection "AccessControlToAuditLogger" {
        from: "LA-GOV-001"
        to: "LA-GOV-003"
    }
    
    connection "DeltaLakeToSQLAnalytics" {
        from: "LA-TGT-003"
        to: "LA-ANLZ-001"
    }
    
    connection "SQLAnalyticsToBIConnector" {
        from: "LA-ANLZ-001"
        to: "LA-ANLZ-003"
    }
    
    connection "MLWorkspaceToLakehouse" {
        from: "LA-ANLZ-002"
        to: "LA-TGT-001"
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTIONS - MONITORING & OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════
    
    connection "LakehouseToObservability" {
        from: "LA-TGT-001"
        to: "LA-MON-001"
    }
    
    connection "ObservabilityToAlertManager" {
        from: "LA-MON-001"
        to: "LA-MON-003"
    }
    
    connection "CostOptimizerToAlertManager" {
        from: "LA-MON-002"
        to: "LA-MON-003"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY - STAKEHOLDER TO SYSTEM REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

trace "STK-001" satisfies "SYS-MIG-005" {
    rationale: "Incremental migration waves reduce TCO by enabling phased decommissioning"
}

trace "STK-001" satisfies "SYS-MON-003" {
    rationale: "Cost tracking and optimization directly supports TCO reduction goals"
}

trace "STK-002" satisfies "SYS-PERF-001" {
    rationale: "Query response time requirement enables real-time analytics capability"
}

trace "STK-002" satisfies "SYS-PERF-004" {
    rationale: "Streaming latency requirement supports real-time analytics"
}

trace "STK-003" satisfies "SYS-MIG-003" {
    rationale: "Data validation framework ensures availability during migration"
}

trace "STK-003" satisfies "SYS-REL-001" {
    rationale: "Automated backup/recovery ensures high availability"
}

trace "STK-004" satisfies "SYS-PERF-003" {
    rationale: "Concurrent user support enables unified platform for data teams"
}

trace "STK-005" satisfies "SYS-MIG-001" {
    rationale: "Bidirectional sync enables zero downtime migration"
}

trace "STK-006" satisfies "SYS-SCALE-001" {
    rationale: "Auto-scaling compute supports 10x growth"
}

trace "STK-006" satisfies "SYS-SCALE-002" {
    rationale: "Elastic storage addresses 100TB to 1PB scalability"
}

trace "STK-007" satisfies "SYS-GOV-001" {
    rationale: "RBAC provides access control for compliance"
}

trace "STK-007" satisfies "SYS-GOV-003" {
    rationale: "Complete data lineage enables required audit trails"
}

trace "STK-008" satisfies "SYS-MIG-002" {
    rationale: "Automated schema mapping accelerates migration timeline"
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY - SYSTEM REQUIREMENTS TO COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

trace "SYS-MIG-001" implements "LA-MIG-004" {
    rationale: "Conflict Resolver implements bidirectional sync"
}

trace "SYS-MIG-002" implements "LA-MIG-002" {
    rationale: "Schema Converter automates schema mapping"
}

trace "SYS-MIG-003" implements "LA-MIG-003" {
    rationale: "Data Validator performs integrity validation"
}

trace "SYS-MIG-004" implements "LA-MIG-001" {
    rationale: "ETL Orchestrator includes rollback functionality"
}

trace "SYS-PERF-001" implements "LA-ANLZ-001" {
    rationale: "SQL Analytics Engine provides fast query response"
}

trace "SYS-PERF-002" implements "LA-TGT-003" {
    rationale: "Delta Lake Storage handles 100TB+ datasets"
}

trace "SYS-PERF-003" implements "LA-TGT-001" {
    rationale: "Databricks Lakehouse supports 500+ concurrent users"
}

trace "SYS-PERF-004" implements "LA-PROC-002" {
    rationale: "Streaming Pipeline Engine delivers low latency"
}

trace "SYS-SCALE-001" implements "LA-TGT-001" {
    rationale: "Databricks Platform provides auto-scaling"
}

trace "SYS-SCALE-002" implements "LA-TGT-003" {
    rationale: "Delta Lake on S3 provides elastic storage"
}

trace "SYS-GOV-001" implements "LA-GOV-001" {
    rationale: "Access Control Manager enforces RBAC"
}

trace "SYS-GOV-002" implements "LA-GOV-001" {
    rationale: "Access Control Manager includes PII detection"
}

trace "SYS-GOV-003" implements "LA-GOV-002" {
    rationale: "Lineage Tracker captures complete lineage"
}

trace "SYS-REL-001" implements "LA-TGT-003" {
    rationale: "Delta Lake provides automated backups"
}

// Physical architecture removed - trace disabled
// trace "SYS-REL-002" implements "PA-CLOUD-001"

trace "SYS-MON-001" implements "LA-MON-001" {
    rationale: "Observability Platform provides monitoring"
}

trace "SYS-MON-002" implements "LA-PROC-003" {
    rationale: "Data Quality Engine tracks quality metrics"
}

trace "SYS-MON-003" implements "LA-MON-002" {
    rationale: "Cost Optimizer tracks and optimizes costs"
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICAL ARCHITECTURE - DEPLOYMENT TOPOLOGY
// ═══════════════════════════════════════════════════════════════════════════

physical_architecture "Data Platform Physical Deployment" {
    // ═══════════════════════════════════════════════════════════════════════
    // CLOUD INFRASTRUCTURE NODES
    // ═══════════════════════════════════════════════════════════════════════
    
    node "AWS Cloud Infrastructure" {
        id: "PA-CLOUD-001"
        processor: "AWS Graviton3 (ARM64)"
        region: "us-east-1"
        availability_zones: "us-east-1a, us-east-1b, us-east-1c"
        
        deploys "LA-TGT-001" {
            partition: "Databricks Control Plane"
            memory: "256GB"
            storage: "10TB NVMe SSD"
        }
        
        deploys "LA-TGT-002" {
            partition: "Unity Catalog Service"
            memory: "128GB"
            criticality: "Critical"
        }
        
        deploys "LA-TGT-003" {
            partition: "Delta Lake Storage Layer"
            storage: "100TB S3 (tiered to Glacier)"
            backup_frequency: "6 hours"
        }
    }
    
    node "Databricks Workspace Cluster" {
        id: "PA-DBX-002"
        processor: "Databricks Managed (AWS i3.xlarge)"
        cluster_type: "Multi-node autoscaling"
        node_range: "2-200 workers"
        
        deploys "LA-PROC-001" {
            partition: "Batch Processing Partition"
            memory: "512GB per worker"
            executor_cores: "16 cores per worker"
        }
        
        deploys "LA-PROC-002" {
            partition: "Streaming Processing Partition"
            memory: "256GB per worker"
            executor_cores: "8 cores per worker"
        }
        
        deploys "LA-PROC-003" {
            partition: "Data Quality Engine"
            memory: "128GB"
        }
    }
    
    node "Governance Control Node" {
        id: "PA-GOV-001"
        processor: "AWS EC2 c6g.4xlarge"
        memory: "32GB"
        storage: "1TB EBS"
        
        deploys "LA-GOV-001" {
            partition: "Access Control Service"
            criticality: "Critical"
            asil: "High"
        }
        
        deploys "LA-GOV-002" {
            partition: "Lineage Tracking Service"
            criticality: "High"
        }
        
        deploys "LA-GOV-003" {
            partition: "Audit Logging Service"
            criticality: "Critical"
            retention_period: "7 years"
        }
    }
    
    node "Integration Gateway Node" {
        id: "PA-INT-001"
        processor: "AWS EC2 m6g.2xlarge"
        memory: "32GB"
        load_balancer: "AWS ALB"
        
        deploys "LA-INT-001" {
            partition: "API Gateway"
            max_connections: "10000"
        }
        
        deploys "LA-INT-002" {
            partition: "Metadata Manager"
            memory: "16GB"
        }
        
        deploys "LA-INT-003" {
            partition: "Workflow Orchestrator"
            memory: "16GB"
        }
    }
    
    node "Analytics Compute Node" {
        id: "PA-ANLZ-001"
        processor: "Databricks SQL Warehouse (Serverless)"
        warehouse_size: "2X-Large"
        auto_suspend: "10 minutes"
        
        deploys "LA-ANLZ-001" {
            partition: "SQL Analytics Engine"
            max_concurrent_queries: "500"
        }
        
        deploys "LA-ANLZ-002" {
            partition: "ML Workspace"
            gpu_support: "NVIDIA A100"
        }
        
        deploys "LA-ANLZ-003" {
            partition: "BI Connector Hub"
            supported_tools: "Tableau, PowerBI, Looker"
        }
    }
    
    node "Monitoring Infrastructure" {
        id: "PA-MON-001"
        processor: "AWS EC2 r6g.xlarge"
        memory: "32GB"
        storage: "5TB EBS (time-series data)"
        
        deploys "LA-MON-001" {
            partition: "Observability Platform"
            metrics_retention: "90 days"
            logs_retention: "30 days"
        }
        
        deploys "LA-MON-002" {
            partition: "Cost Optimizer"
            analysis_frequency: "hourly"
        }
        
        deploys "LA-MON-003" {
            partition: "Alert Manager"
            notification_channels: "PagerDuty, Slack, Email"
        }
    }
    
    node "Migration Engine Node" {
        id: "PA-MIG-001"
        processor: "AWS EC2 c6g.8xlarge"
        memory: "64GB"
        network: "10Gbps dedicated"
        
        deploys "LA-MIG-001" {
            partition: "ETL Orchestrator"
            max_parallel_jobs: "100"
        }
        
        deploys "LA-MIG-002" {
            partition: "Schema Converter"
            supported_sources: "Oracle, Snowflake"
        }
        
        deploys "LA-MIG-003" {
            partition: "Data Validator"
            validation_threads: "32"
        }
        
        deploys "LA-MIG-004" {
            partition: "Conflict Resolver"
            resolution_strategy: "timestamp-based"
        }
    }
    
    node "Source System Gateway" {
        id: "PA-SRC-001"
        processor: "AWS EC2 m6g.4xlarge"
        memory: "64GB"
        network: "Direct Connect to on-premises"
        
        deploys "LA-SRC-001" {
            partition: "Oracle Connector"
            connection_pool: "50 connections"
        }
        
        deploys "LA-SRC-002" {
            partition: "Snowflake Connector"
            connection_pool: "50 connections"
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // PHYSICAL NETWORK LINKS
    // ═══════════════════════════════════════════════════════════════════════
    
    physical_link "AWS VPC Primary Network" {
        id: "PL-001"
        protocol: "TCP/IP over AWS VPC"
        bandwidth: "100Gbps"
        latency: "<1ms"
        connects: ["PA-CLOUD-001", "PA-DBX-002", "PA-GOV-001", "PA-INT-001", "PA-ANLZ-001", "PA-MON-001"]
    }
    
    physical_link "Migration Data Bus" {
        id: "PL-002"
        protocol: "HTTPS/TLS 1.3"
        bandwidth: "10Gbps"
        connects: ["PA-SRC-001", "PA-MIG-001", "PA-CLOUD-001"]
    }
    
    physical_link "Source System Connection" {
        id: "PL-003"
        protocol: "AWS Direct Connect"
        bandwidth: "10Gbps"
        encryption: "MACsec"
        connects: ["PA-SRC-001", "On-Premises Data Center"]
    }
    
    physical_link "Monitoring Telemetry Bus" {
        id: "PL-004"
        protocol: "OpenTelemetry over gRPC"
        connects: ["PA-CLOUD-001", "PA-DBX-002", "PA-GOV-001", "PA-INT-001", "PA-ANLZ-001", "PA-MIG-001", "PA-MON-001"]
    }
    
    physical_link "DR Replication Link" {
        id: "PL-005"
        protocol: "AWS S3 Cross-Region Replication"
        target_region: "us-west-2"
        rpo: "1 hour"
        rto: "4 hours"
        connects: ["PA-CLOUD-001", "PA-CLOUD-DR (us-west-2)"]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ADDITIONAL TRACEABILITY - CLOSING CRITICAL GAPS
// ═══════════════════════════════════════════════════════════════════════════

trace "SYS-REL-002" implements "LA-TGT-003" {
    rationale: "Delta Lake Storage provides DR capability through automated backups, cross-region replication (RPO <1hr, RTO <4hr), deployed on AWS multi-AZ infrastructure (PA-CLOUD-001)"
}

trace "SYS-MIG-005" implements "LA-MIG-001" {
    rationale: "ETL Orchestrator supports incremental wave-based migration with table-level granularity and parallel execution up to 100 concurrent jobs (deployed on PA-MIG-001)"
}

// ═══════════════════════════════════════════════════════════════════════════
// END OF MODEL - 8 Stakeholder Requirements, 19 System Requirements,
// 24 Logical Components, 8 Physical Nodes, 20 Logical Connections,
// 5 Physical Links, 32 Trace Links
// ═══════════════════════════════════════════════════════════════════════════
