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
        compliance_standards: "GDPR, CCPA, SOC2"
        budget_total: "3000000"
        budget_infrastructure: "2000000"
        budget_services: "1000000"
        timeline_months: "12"
        team_size: "9"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STAKEHOLDER REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

requirements stakeholder {
    req "STK-001" "Total Cost of Ownership Reduction" {
        description: "Business must reduce total cost of ownership for data platform by 40% over 3 years through consolidation, automation, and cloud-native architecture"
        stakeholder: "CFO, VP Finance"
        business_value: "Cost savings of $5M annually"
        priority: Critical
        category: "Business"
    }
    
    req "STK-002" "Real-Time and Batch Analytics Support" {
        description: "Platform must support both real-time streaming analytics (latency <1min) and batch analytics for video streaming metrics, viewer behavior, and content performance"
        stakeholder: "Chief Data Officer, Analytics Teams"
        business_value: "Enable data-driven decision making"
        priority: Critical
        category: "Capability"
    }
    
    req "STK-003" "High Availability During Migration" {
        description: "System must maintain 99.9% data availability during migration period with no data loss and seamless failback capability"
        stakeholder: "CTO, VP Engineering"
        business_value: "Business continuity assurance"
        priority: Critical
        category: "Reliability"
    }
    
    req "STK-004" "Unified Platform for Data Teams" {
        description: "Data teams need unified platform combining data engineering (ETL/ELT), data science (ML), and analytics workloads without tool switching"
        stakeholder: "VP Data & Analytics, Data Engineering Leads"
        business_value: "Improved team productivity by 50%"
        priority: High
        category: "Usability"
    }
    
    req "STK-005" "Zero Downtime Migration" {
        description: "Business requires seamless user experience with zero planned downtime during migration, invisible to end users and downstream systems"
        stakeholder: "CEO, VP Product"
        business_value: "No revenue impact during transition"
        priority: Critical
        category: "User Experience"
    }
    
    req "STK-006" "Scalable Architecture for Growth" {
        description: "Organization needs scalable architecture supporting 10x data growth (from 100TB to 1PB) and 5x user growth (from 100 to 500 concurrent users) over 3 years"
        stakeholder: "VP Engineering, Infrastructure Lead"
        business_value: "Future-proof platform"
        priority: High
        category: "Scalability"
    }
    
    req "STK-007" "Full Data Lineage and Audit Trails" {
        description: "Compliance team requires complete data lineage tracking, audit trails for all data access, and automated compliance reporting for GDPR/CCPA"
        stakeholder: "Chief Compliance Officer, Legal"
        business_value: "Regulatory compliance, risk mitigation"
        priority: Critical
        category: "Compliance"
    }
    
    req "STK-008" "12-Month Migration Timeline" {
        description: "Leadership needs migration completed within 12 months with clear milestones: Q1 design, Q2-Q3 incremental migration, Q4 cutover and validation"
        stakeholder: "CEO, Board of Directors"
        business_value: "Time to market for new capabilities"
        priority: Critical
        category: "Schedule"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - DATA MIGRATION
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-MIG-001" "Bidirectional Data Sync" {
        description: "System shall support bidirectional synchronization between source systems (Oracle/Snowflake) and target (Databricks) with conflict detection and resolution"
        derived_from: "STK-005"
        priority: Critical
        verification_method: "Integration testing with conflict scenarios"
        acceptance_criteria: "Bi-directional sync with <5 min latency, 100% conflict detection"
    }
    
    req "SYS-MIG-002" "Automated Schema Mapping" {
        description: "System shall automatically discover and map schemas from Oracle/Snowflake to Databricks Delta Lake format with data type conversion and constraint preservation"
        derived_from: "STK-008"
        priority: High
        verification_method: "Schema comparison testing"
        acceptance_criteria: "95% automated mapping accuracy, manual override capability"
    }
    
    req "SYS-MIG-003" "Data Validation Framework" {
        description: "System shall validate data integrity through row count reconciliation, checksum validation, and sampling-based comparison (1% sample) after each migration batch"
        derived_from: "STK-003"
        priority: Critical
        verification_method: "Data quality testing"
        acceptance_criteria: "99.99% data accuracy, automated validation reports"
    }
    
    req "SYS-MIG-004" "Rollback Procedures" {
        description: "System shall provide automated rollback capability to previous stable state within 15 minutes in case of migration failure or data corruption"
        derived_from: "STK-003"
        priority: Critical
        verification_method: "Disaster recovery testing"
        acceptance_criteria: "Rollback completion <15 min, no data loss"
    }
    
    req "SYS-MIG-005" "Incremental Migration Waves" {
        description: "System shall support incremental migration in waves with table-level granularity, priority-based scheduling, and parallel execution (10 tables simultaneously)"
        derived_from: "STK-008"
        priority: High
        verification_method: "Migration orchestration testing"
        acceptance_criteria: "Wave-based migration, 10 parallel transfers, priority queuing"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - PERFORMANCE
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-PERF-001" "Query Response Time" {
        description: "System shall deliver query response time under 5 seconds for 95th percentile of analytical queries on datasets up to 10TB with optimization recommendations"
        derived_from: "STK-002"
        priority: Critical
        verification_method: "Performance load testing"
        acceptance_criteria: "P95 latency <5s, P99 latency <15s, query optimization"
    }
    
    req "SYS-PERF-002" "Large Dataset Support" {
        description: "System shall efficiently handle datasets exceeding 100TB with columnar storage, Z-ordering, and liquid clustering for optimal query performance"
        derived_from: "STK-006"
        priority: High
        verification_method: "Scalability testing with synthetic 100TB dataset"
        acceptance_criteria: "Linear scalability, query performance maintained"
    }
    
    req "SYS-PERF-003" "Concurrent User Support" {
        description: "System shall support 500+ concurrent users with workload isolation, auto-scaling compute clusters, and query queuing without performance degradation"
        derived_from: "STK-004"
        priority: High
        verification_method: "Load testing with 500 simulated users"
        acceptance_criteria: "500 concurrent users, <10% performance degradation"
    }
    
    req "SYS-PERF-004" "Streaming Latency" {
        description: "System shall process streaming data with end-to-end latency under 60 seconds for 99% of events using structured streaming and auto-loader"
        derived_from: "STK-002"
        priority: Critical
        verification_method: "Streaming pipeline latency monitoring"
        acceptance_criteria: "P99 streaming latency <60s, exactly-once semantics"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - SCALABILITY
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-SCALE-001" "Auto-Scaling Compute" {
        description: "System shall automatically scale compute clusters based on workload demand from 2 to 200 nodes with scale-up time under 2 minutes"
        derived_from: "STK-006"
        priority: High
        verification_method: "Auto-scaling simulation testing"
        acceptance_criteria: "2-200 node scaling, <2 min scale-up, cost-optimized"
    }
    
    req "SYS-SCALE-002" "Elastic Storage" {
        description: "System shall provide elastic cloud storage scaling from 100TB to 1PB+ with automatic tiering (hot/warm/cold) based on access patterns"
        derived_from: "STK-006"
        priority: High
        verification_method: "Storage scalability testing"
        acceptance_criteria: "Elastic scaling to 1PB+, auto-tiering, cost optimization"
    }
    
    req "SYS-SCALE-003" "Multi-Region Support" {
        description: "System shall support multi-region deployment with data replication, regional failover (RTO <5 min), and geo-distributed query execution"
        derived_from: "STK-003"
        priority: Medium
        verification_method: "Multi-region failover testing"
        acceptance_criteria: "Multi-region deployment, failover <5 min, data consistency"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - DATA GOVERNANCE
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-GOV-001" "Role-Based Access Control" {
        description: "System shall enforce granular role-based access control (RBAC) at table, column, and row level with dynamic data masking for PII"
        derived_from: "STK-007"
        priority: Critical
        verification_method: "Security penetration testing"
        acceptance_criteria: "RBAC enforcement, dynamic masking, audit logging"
        safety_level: "High"
    }
    
    req "SYS-GOV-002" "PII Detection and Classification" {
        description: "System shall automatically detect and classify PII data (email, SSN, credit cards) using ML-based scanning and apply appropriate security policies"
        derived_from: "STK-007"
        priority: Critical
        verification_method: "PII detection accuracy testing"
        acceptance_criteria: "95% PII detection accuracy, auto-classification, policy enforcement"
        safety_level: "High"
    }
    
    req "SYS-GOV-003" "Data Classification Tagging" {
        description: "System shall support multi-level data classification (Public, Internal, Confidential, Restricted) with automated tagging and access enforcement"
        derived_from: "STK-007"
        priority: High
        verification_method: "Classification policy testing"
        acceptance_criteria: "4-level classification, auto-tagging, access control integration"
        safety_level: "High"
    }
    
    req "SYS-GOV-004" "Retention Policy Automation" {
        description: "System shall enforce configurable retention policies with automated data archival and deletion based on regulatory requirements (GDPR 7-year retention)"
        derived_from: "STK-007"
        priority: High
        verification_method: "Retention policy compliance audit"
        acceptance_criteria: "Policy-driven retention, automated deletion, audit trails"
        safety_level: "Medium"
    }
    
    req "SYS-GOV-005" "Complete Data Lineage" {
        description: "System shall capture and visualize end-to-end data lineage from source through transformations to consumption with column-level tracking"
        derived_from: "STK-007"
        priority: Critical
        verification_method: "Lineage completeness validation"
        acceptance_criteria: "100% lineage capture, visual DAG, column-level tracking"
        safety_level: "High"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - INTEGRATION & RELIABILITY
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-INT-001" "REST API Integration" {
        description: "System shall expose RESTful APIs for data access, job submission, and metadata management with OAuth 2.0 authentication and rate limiting"
        derived_from: "STK-004"
        priority: High
        verification_method: "API integration testing"
        acceptance_criteria: "REST API, OAuth 2.0, rate limiting, OpenAPI spec"
    }
    
    req "SYS-INT-002" "JDBC/ODBC Connectivity" {
        description: "System shall provide JDBC/ODBC drivers compatible with BI tools (Tableau, PowerBI, Looker) supporting SQL ANSI-2011 standard"
        derived_from: "STK-004"
        priority: High
        verification_method: "BI tool connectivity testing"
        acceptance_criteria: "JDBC/ODBC drivers, BI tool compatibility, SQL ANSI compliance"
    }
    
    req "SYS-INT-003" "CI/CD Pipeline Integration" {
        description: "System shall integrate with CI/CD pipelines (GitHub Actions, Jenkins) for automated testing, deployment, and rollback of data pipelines"
        derived_from: "STK-001"
        priority: Medium
        verification_method: "CI/CD automation testing"
        acceptance_criteria: "CI/CD integration, automated testing, rollback capability"
    }
    
    req "SYS-REL-001" "Automated Backup and Recovery" {
        description: "System shall perform automated incremental backups every 6 hours with point-in-time recovery capability and 30-day retention"
        derived_from: "STK-003"
        priority: Critical
        verification_method: "Backup/recovery testing"
        acceptance_criteria: "6-hour backup frequency, PITR, 30-day retention"
        safety_level: "High"
    }
    
    req "SYS-REL-002" "Disaster Recovery SLA" {
        description: "System shall meet disaster recovery SLA with Recovery Point Objective (RPO) under 1 hour and Recovery Time Objective (RTO) under 4 hours"
        derived_from: "STK-003"
        priority: Critical
        verification_method: "DR drill execution"
        acceptance_criteria: "RPO <1hr, RTO <4hr, automated failover"
        safety_level: "High"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS - MONITORING & OBSERVABILITY
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    req "SYS-MON-001" "Real-Time Pipeline Monitoring" {
        description: "System shall provide real-time monitoring of data pipelines with alerts for failures, SLA breaches, and data quality issues via Slack/PagerDuty"
        derived_from: "STK-001"
        priority: High
        verification_method: "Monitoring dashboard validation"
        acceptance_criteria: "Real-time monitoring, alerting, dashboard, SLA tracking"
    }
    
    req "SYS-MON-002" "Data Quality Metrics" {
        description: "System shall track data quality metrics (completeness, accuracy, consistency, timeliness) with automated anomaly detection and root cause analysis"
        derived_from: "STK-003"
        priority: High
        verification_method: "Data quality testing"
        acceptance_criteria: "Quality metrics, anomaly detection, root cause analysis"
    }
    
    req "SYS-MON-003" "Cost Tracking and Optimization" {
        description: "System shall track cloud costs by workload, team, and project with cost optimization recommendations and budget alerts at 80% threshold"
        derived_from: "STK-001"
        priority: Medium
        verification_method: "Cost tracking validation"
        acceptance_criteria: "Cost tracking, optimization recommendations, budget alerts"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - SOURCE DATA LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Oracle Database Subsystem" {
        id: "LA-SRC-001"
        description: "Legacy Oracle database containing customer data, transactions, and operational metrics (50TB)"
        category: "SourceSystem"
        technology: "Oracle 19c RAC"
        data_volume: "50TB"
        criticality: "High"
        
        function "Customer Data Management" {
            id: "LA-SRC-001-F1"
            description: "Manages customer profiles, preferences, and viewing history"
            inputs: ["customer_events"]
            outputs: ["customer_records"]
        }
        
        function "Transaction Processing" {
            id: "LA-SRC-001-F2"
            description: "Processes subscription transactions and billing records"
            inputs: ["transaction_events"]
            outputs: ["transaction_records"]
        }
    }
    
    component "Snowflake Warehouse Subsystem" {
        id: "LA-SRC-002"
        description: "Existing Snowflake data warehouse with analytics workloads and aggregated metrics (50TB)"
        category: "SourceSystem"
        technology: "Snowflake Enterprise"
        data_volume: "50TB"
        criticality: "High"
        
        function "Analytics Data Serving" {
            id: "LA-SRC-002-F1"
            description: "Serves aggregated metrics for BI dashboards and reports"
            inputs: ["query_requests"]
            outputs: ["analytics_results"]
        }
        
        function "Historical Data Storage" {
            id: "LA-SRC-002-F2"
            description: "Stores 3-year historical data for trend analysis"
            inputs: ["historical_data"]
            outputs: ["historical_queries"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - MIGRATION ENGINE
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "ETL Orchestrator" {
        id: "LA-MIG-001"
        description: "Orchestrates end-to-end migration workflows with wave-based execution, dependency management, and parallel processing"
        category: "MigrationEngine"
        technology: "Apache Airflow on Databricks Workflows"
        safety_level: "High"
        criticality: "Critical"
        
        function "Migration Wave Scheduling" {
            id: "LA-MIG-001-F1"
            description: "Schedules tables for migration in priority-based waves with dependency resolution"
            inputs: ["migration_plan", "table_dependencies"]
            outputs: ["scheduled_jobs"]
        }
        
        function "Parallel Execution Management" {
            id: "LA-MIG-001-F2"
            description: "Manages parallel execution of up to 10 table migrations with resource allocation"
            inputs: ["scheduled_jobs", "cluster_capacity"]
            outputs: ["execution_status"]
        }
        
        function "Failure Recovery" {
            id: "LA-MIG-001-F3"
            description: "Handles migration failures with automatic retry, rollback, and alerting"
            inputs: ["failure_events"]
            outputs: ["recovery_actions"]
        }
    }
    
    component "Schema Converter" {
        id: "LA-MIG-002"
        description: "Automatically converts Oracle/Snowflake schemas to Databricks Delta Lake format with data type mapping and constraint preservation"
        category: "MigrationEngine"
        technology: "Custom Python + Databricks SQL"
        safety_level: "High"
        criticality: "High"
        
        function "Schema Discovery" {
            id: "LA-MIG-002-F1"
            description: "Discovers source schemas, data types, constraints, and relationships"
            inputs: ["source_metadata"]
            outputs: ["discovered_schemas"]
        }
        
        function "Type Mapping Engine" {
            id: "LA-MIG-002-F2"
            description: "Maps Oracle/Snowflake data types to Delta Lake compatible types"
            inputs: ["discovered_schemas", "mapping_rules"]
            outputs: ["converted_schemas"]
        }
        
        function "Constraint Translation" {
            id: "LA-MIG-002-F3"
            description: "Translates primary keys, foreign keys, and check constraints to Delta Lake"
            inputs: ["source_constraints"]
            outputs: ["delta_constraints"]
        }
    }
    
    component "Data Validator" {
        id: "LA-MIG-003"
        description: "Validates migrated data through row count reconciliation, checksum comparison, and sampling-based validation (1% sample)"
        category: "MigrationEngine"
        technology: "PySpark + Great Expectations"
        safety_level: "Critical"
        criticality: "Critical"
        
        function "Row Count Reconciliation" {
            id: "LA-MIG-003-F1"
            description: "Compares row counts between source and target with detailed discrepancy reporting"
            inputs: ["source_counts", "target_counts"]
            outputs: ["count_report"]
        }
        
        function "Checksum Validation" {
            id: "LA-MIG-003-F2"
            description: "Computes and compares MD5 checksums for data integrity verification"
            inputs: ["source_data", "target_data"]
            outputs: ["checksum_results"]
        }
        
        function "Sampling Comparison" {
            id: "LA-MIG-003-F3"
            description: "Performs deep comparison on 1% sample with column-by-column validation"
            inputs: ["source_sample", "target_sample"]
            outputs: ["validation_report"]
        }
    }
    
    component "Conflict Resolver" {
        id: "LA-MIG-004"
        description: "Detects and resolves data conflicts during bidirectional sync with configurable resolution strategies"
        category: "MigrationEngine"
        technology: "Custom Python + Delta Lake Merge"
        safety_level: "High"
        criticality: "High"
        
        function "Conflict Detection" {
            id: "LA-MIG-004-F1"
            description: "Identifies conflicts in bidirectional sync using timestamp and version comparison"
            inputs: ["source_changes", "target_changes"]
            outputs: ["detected_conflicts"]
        }
        
        function "Resolution Strategy Application" {
            id: "LA-MIG-004-F2"
            description: "Applies resolution rules: source-wins, target-wins, latest-wins, or manual review"
            inputs: ["detected_conflicts", "resolution_rules"]
            outputs: ["resolved_data"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - TARGET PLATFORM LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Databricks Lakehouse Platform" {
        id: "LA-TGT-001"
        description: "Unified lakehouse platform combining data lake and data warehouse capabilities with Delta Lake storage engine"
        category: "TargetPlatform"
        technology: "Databricks Lakehouse"
        safety_level: "High"
        criticality: "Critical"
        
        function "Unified Query Engine" {
            id: "LA-TGT-001-F1"
            description: "Executes SQL, Python, and Scala queries with Photon vectorized execution"
            inputs: ["query_requests"]
            outputs: ["query_results"]
        }
        
        function "Compute Cluster Management" {
            id: "LA-TGT-001-F2"
            description: "Manages auto-scaling clusters for interactive and batch workloads"
            inputs: ["workload_requests"]
            outputs: ["cluster_allocation"]
        }
    }
    
    component "Unity Catalog" {
        id: "LA-TGT-002"
        description: "Unified governance solution for data and AI assets with fine-grained access control and centralized metadata"
        category: "Governance"
        technology: "Databricks Unity Catalog"
        safety_level: "Critical"
        criticality: "Critical"
        
        function "Metadata Management" {
            id: "LA-TGT-002-F1"
            description: "Centralized metadata repository for catalogs, schemas, tables, and columns"
            inputs: ["metadata_updates"]
            outputs: ["metadata_catalog"]
        }
        
        function "Access Control Enforcement" {
            id: "LA-TGT-002-F2"
            description: "Enforces RBAC policies at catalog, schema, table, and column levels"
            inputs: ["access_requests", "rbac_policies"]
            outputs: ["access_decisions"]
        }
        
        function "Data Lineage Tracking" {
            id: "LA-TGT-002-F3"
            description: "Automatically captures column-level lineage for all operations"
            inputs: ["query_executions"]
            outputs: ["lineage_graph"]
        }
    }
    
    component "Delta Lake Storage" {
        id: "LA-TGT-003"
        description: "ACID-compliant storage layer with versioning, time travel, and schema evolution"
        category: "Storage"
        technology: "Delta Lake 3.0"
        safety_level: "High"
        criticality: "Critical"
        
        function "ACID Transaction Processing" {
            id: "LA-TGT-003-F1"
            description: "Provides ACID guarantees for all read and write operations"
            inputs: ["write_operations"]
            outputs: ["committed_transactions"]
        }
        
        function "Time Travel Queries" {
            id: "LA-TGT-003-F2"
            description: "Enables querying historical versions of data using AS OF syntax"
            inputs: ["time_travel_queries"]
            outputs: ["historical_data"]
        }
        
        function "Schema Evolution" {
            id: "LA-TGT-003-F3"
            description: "Supports schema changes without rewriting existing data"
            inputs: ["schema_changes"]
            outputs: ["evolved_schema"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - DATA PROCESSING LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Batch Pipeline Engine" {
        id: "LA-PROC-001"
        description: "Processes large-scale batch workloads for ETL, aggregations, and model training with medallion architecture (bronze/silver/gold)"
        category: "Processing"
        technology: "PySpark + Delta Live Tables"
        safety_level: "Medium"
        criticality: "High"
        
        function "Bronze Layer Ingestion" {
            id: "LA-PROC-001-F1"
            description: "Ingests raw data from sources with minimal transformation"
            inputs: ["raw_data"]
            outputs: ["bronze_tables"]
        }
        
        function "Silver Layer Transformation" {
            id: "LA-PROC-001-F2"
            description: "Cleanses, deduplicates, and standardizes data"
            inputs: ["bronze_tables"]
            outputs: ["silver_tables"]
        }
        
        function "Gold Layer Aggregation" {
            id: "LA-PROC-001-F3"
            description: "Creates business-level aggregates and curated datasets"
            inputs: ["silver_tables"]
            outputs: ["gold_tables"]
        }
    }
    
    component "Streaming Pipeline Engine" {
        id: "LA-PROC-002"
        description: "Processes real-time streaming data with exactly-once semantics and sub-minute latency for video streaming events"
        category: "Processing"
        technology: "Spark Structured Streaming + Auto Loader"
        safety_level: "Medium"
        criticality: "High"
        
        function "Stream Ingestion" {
            id: "LA-PROC-002-F1"
            description: "Ingests streaming events from Kafka/Kinesis with checkpointing"
            inputs: ["streaming_events"]
            outputs: ["streaming_batches"]
        }
        
        function "Real-Time Transformations" {
            id: "LA-PROC-002-F2"
            description: "Applies transformations, windowing, and aggregations on streams"
            inputs: ["streaming_batches"]
            outputs: ["transformed_streams"]
        }
        
        function "Streaming Sink Management" {
            id: "LA-PROC-002-F3"
            description: "Writes processed streams to Delta tables with exactly-once guarantees"
            inputs: ["transformed_streams"]
            outputs: ["delta_sink"]
        }
    }
    
    component "Data Quality Engine" {
        id: "LA-PROC-003"
        description: "Enforces data quality rules, detects anomalies, and quarantines invalid data with alerting"
        category: "DataQuality"
        technology: "Great Expectations + Custom PySpark"
        safety_level: "High"
        criticality: "High"
        
        function "Quality Rule Enforcement" {
            id: "LA-PROC-003-F1"
            description: "Validates data against configurable quality rules (completeness, uniqueness, ranges)"
            inputs: ["incoming_data", "quality_rules"]
            outputs: ["validation_results"]
        }
        
        function "Anomaly Detection" {
            id: "LA-PROC-003-F2"
            description: "Detects statistical anomalies using ML-based outlier detection"
            inputs: ["data_metrics", "historical_patterns"]
            outputs: ["detected_anomalies"]
        }
        
        function "Quarantine Management" {
            id: "LA-PROC-003-F3"
            description: "Quarantines invalid records for manual review and correction"
            inputs: ["validation_failures"]
            outputs: ["quarantine_table"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - GOVERNANCE LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Access Control Manager" {
        id: "LA-GOV-001"
        description: "Manages fine-grained access control with RBAC, ABAC, and dynamic data masking for PII protection"
        category: "Governance"
        technology: "Unity Catalog + Custom Python"
        safety_level: "Critical"
        criticality: "Critical"
        
        function "RBAC Policy Enforcement" {
            id: "LA-GOV-001-F1"
            description: "Enforces role-based policies at catalog, schema, table, column, and row levels"
            inputs: ["user_requests", "rbac_policies"]
            outputs: ["access_grants"]
        }
        
        function "Dynamic Data Masking" {
            id: "LA-GOV-001-F2"
            description: "Automatically masks PII fields based on user roles and data classification"
            inputs: ["query_results", "masking_rules"]
            outputs: ["masked_data"]
        }
        
        function "Audit Logging" {
            id: "LA-GOV-001-F3"
            description: "Logs all data access attempts with user, timestamp, and data accessed"
            inputs: ["access_events"]
            outputs: ["audit_logs"]
        }
    }
    
    component "Lineage Tracker" {
        id: "LA-GOV-002"
        description: "Captures end-to-end data lineage from source systems through transformations to consumption"
        category: "Governance"
        technology: "Unity Catalog Lineage + OpenLineage"
        safety_level: "High"
        criticality: "High"
        
        function "Automatic Lineage Capture" {
            id: "LA-GOV-002-F1"
            description: "Automatically captures lineage for Spark SQL, Python, and notebook operations"
            inputs: ["query_plans"]
            outputs: ["lineage_metadata"]
        }
        
        function "Lineage Visualization" {
            id: "LA-GOV-002-F2"
            description: "Generates interactive DAG visualization of data flows"
            inputs: ["lineage_metadata"]
            outputs: ["lineage_graph"]
        }
        
        function "Impact Analysis" {
            id: "LA-GOV-002-F3"
            description: "Analyzes upstream and downstream impact of schema or data changes"
            inputs: ["change_requests", "lineage_graph"]
            outputs: ["impact_report"]
        }
    }
    
    component "Audit Logger" {
        id: "LA-GOV-003"
        description: "Centralized audit logging for compliance with GDPR, CCPA, SOC2 requirements"
        category: "Governance"
        technology: "Databricks Audit Logs + Splunk"
        safety_level: "Critical"
        criticality: "Critical"
        
        function "Event Collection" {
            id: "LA-GOV-003-F1"
            description: "Collects audit events from all platform components"
            inputs: ["platform_events"]
            outputs: ["audit_events"]
        }
        
        function "Compliance Reporting" {
            id: "LA-GOV-003-F2"
            description: "Generates compliance reports for GDPR access requests and CCPA deletions"
            inputs: ["audit_events", "compliance_queries"]
            outputs: ["compliance_reports"]
        }
        
        function "Long-Term Retention" {
            id: "LA-GOV-003-F3"
            description: "Archives audit logs to cold storage with 7-year retention"
            inputs: ["audit_events"]
            outputs: ["archived_logs"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - INTEGRATION LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "API Gateway" {
        id: "LA-INT-001"
        description: "RESTful API gateway for data access, job submission, and metadata queries with OAuth 2.0 and rate limiting"
        category: "Integration"
        technology: "Databricks REST API + API Gateway"
        safety_level: "High"
        criticality: "High"
        
        function "Authentication & Authorization" {
            id: "LA-INT-001-F1"
            description: "Handles OAuth 2.0 authentication and JWT token validation"
            inputs: ["api_requests", "credentials"]
            outputs: ["auth_tokens"]
        }
        
        function "Rate Limiting" {
            id: "LA-INT-001-F2"
            description: "Enforces rate limits per user/application to prevent abuse"
            inputs: ["api_requests"]
            outputs: ["rate_limit_decisions"]
        }
        
        function "Request Routing" {
            id: "LA-INT-001-F3"
            description: "Routes API requests to appropriate backend services"
            inputs: ["validated_requests"]
            outputs: ["routed_requests"]
        }
    }
    
    component "Metadata Manager" {
        id: "LA-INT-002"
        description: "Centralized metadata management for technical, business, and operational metadata"
        category: "Integration"
        technology: "Unity Catalog + Custom Metadata API"
        safety_level: "Medium"
        criticality: "High"
        
        function "Technical Metadata" {
            id: "LA-INT-002-F1"
            description: "Manages schema definitions, data types, and constraints"
            inputs: ["schema_updates"]
            outputs: ["technical_catalog"]
        }
        
        function "Business Metadata" {
            id: "LA-INT-002-F2"
            description: "Manages business glossary, data owners, and descriptions"
            inputs: ["business_definitions"]
            outputs: ["business_catalog"]
        }
        
        function "Operational Metadata" {
            id: "LA-INT-002-F3"
            description: "Tracks data freshness, quality scores, and usage statistics"
            inputs: ["operational_metrics"]
            outputs: ["operational_catalog"]
        }
    }
    
    component "Workflow Orchestrator" {
        id: "LA-INT-003"
        description: "Orchestrates complex multi-step workflows across data pipelines, ML training, and analytics jobs"
        category: "Integration"
        technology: "Databricks Workflows + Apache Airflow"
        safety_level: "Medium"
        criticality: "High"
        
        function "DAG Definition & Execution" {
            id: "LA-INT-003-F1"
            description: "Defines and executes directed acyclic graphs of tasks"
            inputs: ["workflow_definitions"]
            outputs: ["execution_plans"]
        }
        
        function "Dependency Management" {
            id: "LA-INT-003-F2"
            description: "Manages task dependencies and execution order"
            inputs: ["task_dependencies"]
            outputs: ["execution_sequence"]
        }
        
        function "Failure Handling" {
            id: "LA-INT-003-F3"
            description: "Handles task failures with retry logic and alerting"
            inputs: ["task_failures"]
            outputs: ["recovery_actions"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - ANALYTICS LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "SQL Analytics Engine" {
        id: "LA-ANLZ-001"
        description: "SQL-based analytics engine optimized for BI workloads with result caching and query acceleration"
        category: "Analytics"
        technology: "Databricks SQL + Photon Engine"
        safety_level: "Low"
        criticality: "High"
        
        function "SQL Query Execution" {
            id: "LA-ANLZ-001-F1"
            description: "Executes SQL queries with Photon vectorized engine for performance"
            inputs: ["sql_queries"]
            outputs: ["query_results"]
        }
        
        function "Result Caching" {
            id: "LA-ANLZ-001-F2"
            description: "Caches query results for repeated queries to improve performance"
            inputs: ["query_results"]
            outputs: ["cached_results"]
        }
        
        function "Query Profiling" {
            id: "LA-ANLZ-001-F3"
            description: "Provides query execution plans and optimization recommendations"
            inputs: ["executed_queries"]
            outputs: ["profiling_reports"]
        }
    }
    
    component "ML Workspace" {
        id: "LA-ANLZ-002"
        description: "Machine learning workspace with notebook environment, MLflow tracking, and model serving"
        category: "Analytics"
        technology: "Databricks ML Runtime + MLflow"
        safety_level: "Low"
        criticality: "Medium"
        
        function "Notebook Environment" {
            id: "LA-ANLZ-002-F1"
            description: "Interactive notebooks for data exploration and model development"
            inputs: ["notebook_code"]
            outputs: ["notebook_results"]
        }
        
        function "MLflow Experiment Tracking" {
            id: "LA-ANLZ-002-F2"
            description: "Tracks ML experiments, parameters, metrics, and artifacts"
            inputs: ["experiment_runs"]
            outputs: ["experiment_metadata"]
        }
        
        function "Model Registry & Serving" {
            id: "LA-ANLZ-002-F3"
            description: "Registers, versions, and serves ML models with REST endpoints"
            inputs: ["trained_models"]
            outputs: ["model_endpoints"]
        }
    }
    
    component "BI Connector Hub" {
        id: "LA-ANLZ-003"
        description: "Connectivity layer for BI tools (Tableau, PowerBI, Looker) with semantic layer and performance optimization"
        category: "Analytics"
        technology: "JDBC/ODBC + Partner Connect"
        safety_level: "Low"
        criticality: "Medium"
        
        function "JDBC/ODBC Gateway" {
            id: "LA-ANLZ-003-F1"
            description: "Provides JDBC/ODBC connectivity for BI tool integration"
            inputs: ["bi_tool_queries"]
            outputs: ["jdbc_results"]
        }
        
        function "Semantic Layer" {
            id: "LA-ANLZ-003-F2"
            description: "Business-friendly semantic layer with metrics and dimensions"
            inputs: ["business_queries"]
            outputs: ["semantic_results"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE - MONITORING & OPERATIONS LAYER
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    component "Observability Platform" {
        id: "LA-MON-001"
        description: "Centralized observability with metrics, logs, and traces for all platform components"
        category: "Monitoring"
        technology: "Datadog + Databricks System Tables"
        safety_level: "Medium"
        criticality: "High"
        
        function "Metrics Collection" {
            id: "LA-MON-001-F1"
            description: "Collects performance metrics, resource utilization, and job statistics"
            inputs: ["platform_metrics"]
            outputs: ["metrics_database"]
        }
        
        function "Log Aggregation" {
            id: "LA-MON-001-F2"
            description: "Aggregates logs from clusters, jobs, and applications"
            inputs: ["application_logs"]
            outputs: ["centralized_logs"]
        }
        
        function "Distributed Tracing" {
            id: "LA-MON-001-F3"
            description: "Traces requests across distributed components for performance debugging"
            inputs: ["trace_spans"]
            outputs: ["trace_visualizations"]
        }
    }
    
    component "Cost Optimizer" {
        id: "LA-MON-002"
        description: "Cloud cost optimization with workload-based recommendations, idle cluster detection, and budget management"
        category: "Operations"
        technology: "Databricks Billing + Custom Analytics"
        safety_level: "Low"
        criticality: "Medium"
        
        function "Cost Attribution" {
            id: "LA-MON-002-F1"
            description: "Attributes costs to teams, projects, and workloads"
            inputs: ["usage_data", "cost_data"]
            outputs: ["cost_reports"]
        }
        
        function "Optimization Recommendations" {
            id: "LA-MON-002-F2"
            description: "Recommends cluster sizing, spot instance usage, and storage tiering"
            inputs: ["usage_patterns"]
            outputs: ["optimization_suggestions"]
        }
        
        function "Budget Alerts" {
            id: "LA-MON-002-F3"
            description: "Alerts when spending exceeds 80% of budget thresholds"
            inputs: ["current_spend", "budget_limits"]
            outputs: ["budget_alerts"]
        }
    }
    
    component "Alert Manager" {
        id: "LA-MON-003"
        description: "Intelligent alerting system with anomaly detection, alert routing, and on-call management"
        category: "Monitoring"
        technology: "PagerDuty + Databricks SQL Alerts"
        safety_level: "Medium"
        criticality: "High"
        
        function "Alert Rule Engine" {
            id: "LA-MON-003-F1"
            description: "Evaluates alert rules based on metrics, logs, and data quality checks"
            inputs: ["alert_rules", "monitoring_data"]
            outputs: ["triggered_alerts"]
        }
        
        function "Smart Routing" {
            id: "LA-MON-003-F2"
            description: "Routes alerts to appropriate teams based on severity and component"
            inputs: ["triggered_alerts"]
            outputs: ["routed_notifications"]
        }
        
        function "Alert Suppression" {
            id: "LA-MON-003-F3"
            description: "Suppresses duplicate alerts and implements maintenance windows"
            inputs: ["triggered_alerts", "suppression_rules"]
            outputs: ["filtered_alerts"]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICAL ARCHITECTURE - CLOUD INFRASTRUCTURE
// ═══════════════════════════════════════════════════════════════════════════

architecture physical {
    node "AWS Cloud Infrastructure" {
        id: "PA-CLOUD-001"
        description: "Primary cloud infrastructure on AWS in us-east-1 region with multi-AZ deployment"
        processor: "AWS Graviton3 (ARM64)"
        technology: "AWS EC2, S3, VPC"
        region: "us-east-1"
        availability_zones: "us-east-1a, us-east-1b, us-east-1c"
        
        deploys "LA-TGT-001"
        deploys "LA-TGT-002"
        deploys "LA-TGT-003"
    }
    
    node "Databricks Control Plane" {
        id: "PA-DBX-001"
        description: "Databricks managed control plane for workspace management, job scheduling, and cluster orchestration"
        processor: "Databricks Managed"
        technology: "Databricks Platform"
        deployment_model: "SaaS"
        
        deploys "LA-INT-003"
        deploys "LA-MON-001"
    }
    
    node "Databricks Data Plane" {
        id: "PA-DBX-002"
        description: "Customer-managed data plane with compute clusters deployed in customer VPC"
        processor: "AWS EC2 (i3.xlarge to i3.16xlarge)"
        technology: "Spark Clusters"
        deployment_model: "VPC Peering"
        auto_scaling: "2-200 nodes"
        
        deploys "LA-PROC-001"
        deploys "LA-PROC-002"
        deploys "LA-PROC-003"
        deploys "LA-ANLZ-001"
    }
    
    node "Delta Lake Storage" {
        id: "PA-STORAGE-001"
        description: "S3-based storage for Delta Lake tables with intelligent tiering"
        processor: "AWS S3"
        technology: "S3 Intelligent Tiering"
        capacity: "100TB initial, 1PB scalable"
        storage_classes: "S3 Standard, S3 IA, S3 Glacier"
        
        deploys "LA-TGT-003"
    }
    
    node "Unity Catalog Metastore" {
        id: "PA-CATALOG-001"
        description: "Centralized metastore for Unity Catalog metadata"
        processor: "Databricks Managed"
        technology: "Unity Catalog"
        deployment_model: "Regional"
        
        deploys "LA-TGT-002"
        deploys "LA-GOV-002"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICAL ARCHITECTURE - NETWORKING & SECURITY
// ═══════════════════════════════════════════════════════════════════════════

architecture physical {
    node "VPC Network" {
        id: "PA-NET-001"
        description: "Isolated VPC with private subnets for data plane and public subnets for NAT gateways"
        processor: "AWS VPC"
        technology: "VPC, Subnets, Security Groups"
        cidr_block: "10.0.0.0/16"
        
        deploys "LA-INT-001"
    }
    
    node "AWS PrivateLink" {
        id: "PA-NET-002"
        description: "Private connectivity between Databricks and AWS services without internet exposure"
        processor: "AWS PrivateLink"
        technology: "VPC Endpoints"
        
        deploys "LA-INT-001"
    }
    
    node "AWS Direct Connect" {
        id: "PA-NET-003"
        description: "Dedicated network connection from on-premise Oracle to AWS for data migration"
        processor: "AWS Direct Connect"
        technology: "1 Gbps Dedicated Connection"
        bandwidth: "1 Gbps"
    }
    
    node "IAM Security Layer" {
        id: "PA-SEC-001"
        description: "AWS IAM for cloud resource access control and Databricks identity federation"
        processor: "AWS IAM"
        technology: "IAM Roles, SAML 2.0, SCIM"
        
        deploys "LA-GOV-001"
    }
    
    node "Encryption Services" {
        id: "PA-SEC-002"
        description: "Encryption at rest using AWS KMS and in-transit using TLS 1.3"
        processor: "AWS KMS"
        technology: "KMS, TLS 1.3"
        encryption_at_rest: "AES-256"
        encryption_in_transit: "TLS 1.3"
        
        deploys "LA-GOV-001"
    }
    
    node "Secrets Manager" {
        id: "PA-SEC-003"
        description: "Centralized secrets management for database credentials and API keys"
        processor: "AWS Secrets Manager"
        technology: "Secrets Manager, Databricks Secrets"
        rotation_policy: "90 days"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICAL ARCHITECTURE - DEVOPS & CI/CD
// ═══════════════════════════════════════════════════════════════════════════

architecture physical {
    node "Git Repository" {
        id: "PA-DEV-001"
        description: "GitHub Enterprise for source code, notebooks, and infrastructure as code"
        processor: "GitHub Enterprise"
        technology: "Git, GitHub Actions"
        
        deploys "LA-INT-003"
    }
    
    node "CI/CD Pipeline" {
        id: "PA-DEV-002"
        description: "Automated CI/CD pipelines for testing and deploying data pipelines and notebooks"
        processor: "GitHub Actions"
        technology: "GitHub Actions, Databricks CLI"
        
        deploys "LA-INT-003"
    }
    
    node "Infrastructure as Code" {
        id: "PA-DEV-003"
        description: "Terraform for infrastructure provisioning and Databricks resource management"
        processor: "Terraform Cloud"
        technology: "Terraform, Databricks Terraform Provider"
    }
    
    node "Artifact Repository" {
        id: "PA-DEV-004"
        description: "Artifact storage for Python wheels, JAR files, and Docker images"
        processor: "AWS S3, ECR"
        technology: "S3, Elastic Container Registry"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTIONS - MIGRATION DATA FLOW
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    connection "OracleToMigrationEngine" {
        from: "LA-SRC-001"
        to: "LA-MIG-001"
        protocol: "JDBC"
        data_format: "Relational"
        bandwidth: "1 Gbps"
        description: "Bulk data extraction from Oracle using JDBC with parallel connections"
    }
    
    connection "SnowflakeToMigrationEngine" {
        from: "LA-SRC-002"
        to: "LA-MIG-001"
        protocol: "Snowpipe"
        data_format: "Parquet"
        bandwidth: "1 Gbps"
        description: "Snowflake UNLOAD to S3 staging followed by Databricks COPY INTO"
    }
    
    connection "MigrationEngineToSchemaConverter" {
        from: "LA-MIG-001"
        to: "LA-MIG-002"
        protocol: "Internal API"
        data_format: "Metadata JSON"
        description: "Schema metadata passed to converter for type mapping"
    }
    
    connection "SchemaConverterToValidator" {
        from: "LA-MIG-002"
        to: "LA-MIG-003"
        protocol: "Internal API"
        data_format: "Schema DDL"
        description: "Converted schemas passed to validator for verification"
    }
    
    connection "MigrationEngineToDeltaLake" {
        from: "LA-MIG-001"
        to: "LA-TGT-003"
        protocol: "Delta Lake API"
        data_format: "Delta Parquet"
        description: "Migrated data written to Delta Lake tables"
    }
    
    connection "ValidatorBidirectionalSync" {
        from: "LA-MIG-003"
        to: "LA-MIG-004"
        protocol: "Internal API"
        data_format: "Validation Reports"
        bidirectional: true
        description: "Validation results trigger conflict resolution"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTIONS - DATA PROCESSING FLOW
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    connection "DeltaLakeToBatchPipeline" {
        from: "LA-TGT-003"
        to: "LA-PROC-001"
        protocol: "Delta Lake Read API"
        data_format: "Delta Parquet"
        description: "Batch pipelines read from Delta Lake bronze tables"
    }
    
    connection "BatchPipelineToDataQuality" {
        from: "LA-PROC-001"
        to: "LA-PROC-003"
        protocol: "Internal API"
        data_format: "DataFrame"
        description: "Data quality checks applied during transformations"
    }
    
    connection "StreamingPipelineToDeltaLake" {
        from: "LA-PROC-002"
        to: "LA-TGT-003"
        protocol: "Structured Streaming"
        data_format: "Delta Streaming"
        description: "Streaming sink writes to Delta Lake with exactly-once semantics"
    }
    
    connection "DataQualityToQuarantine" {
        from: "LA-PROC-003"
        to: "LA-TGT-003"
        protocol: "Delta Lake Write API"
        data_format: "Delta Parquet"
        description: "Invalid records written to quarantine tables"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTIONS - GOVERNANCE & ANALYTICS FLOW
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    connection "UnityCatalogToAccessControl" {
        from: "LA-TGT-002"
        to: "LA-GOV-001"
        protocol: "Internal API"
        data_format: "Access Policies"
        description: "Unity Catalog enforces access control policies"
    }
    
    connection "UnityCatalogToLineageTracker" {
        from: "LA-TGT-002"
        to: "LA-GOV-002"
        protocol: "Internal API"
        data_format: "Lineage Metadata"
        description: "Unity Catalog captures lineage automatically"
    }
    
    connection "AccessControlToAuditLogger" {
        from: "LA-GOV-001"
        to: "LA-GOV-003"
        protocol: "Event Stream"
        data_format: "Audit Events"
        description: "All access decisions logged for compliance"
    }
    
    connection "DeltaLakeToSQLAnalytics" {
        from: "LA-TGT-003"
        to: "LA-ANLZ-001"
        protocol: "Databricks SQL"
        data_format: "Delta Tables"
        description: "SQL analytics queries Delta Lake gold tables"
    }
    
    connection "SQLAnalyticsToBIConnector" {
        from: "LA-ANLZ-001"
        to: "LA-ANLZ-003"
        protocol: "JDBC/ODBC"
        data_format: "Result Sets"
        description: "BI tools connect via JDBC/ODBC"
    }
    
    connection "MLWorkspaceToModelRegistry" {
        from: "LA-ANLZ-002"
        to: "LA-TGT-001"
        protocol: "MLflow API"
        data_format: "ML Models"
        description: "ML models registered and versioned in MLflow"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTIONS - MONITORING & OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    connection "AllComponentsToObservability" {
        from: "LA-TGT-001"
        to: "LA-MON-001"
        protocol: "Metrics API"
        data_format: "Time Series"
        description: "All components send metrics to observability platform"
    }
    
    connection "ObservabilityToAlertManager" {
        from: "LA-MON-001"
        to: "LA-MON-003"
        protocol: "Webhooks"
        data_format: "Alert Payloads"
        description: "Metrics trigger alerts based on thresholds"
    }
    
    connection "CostOptimizerToAlertManager" {
        from: "LA-MON-002"
        to: "LA-MON-003"
        protocol: "Internal API"
        data_format: "Cost Alerts"
        description: "Budget threshold breaches trigger alerts"
    }
    
    connection "AlertManagerToSlack" {
        from: "LA-MON-003"
        to: "LA-INT-001"
        protocol: "Webhook"
        data_format: "Slack Messages"
        description: "Alerts routed to Slack channels and PagerDuty"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY - STAKEHOLDER TO SYSTEM REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

trace "STK-001" satisfies "SYS-MIG-005" {
    rationale: "Incremental migration waves reduce TCO by enabling phased decommissioning of legacy systems"
}

trace "STK-001" satisfies "SYS-MON-003" {
    rationale: "Cost tracking and optimization directly supports TCO reduction goals"
}

trace "STK-002" satisfies "SYS-PERF-001" {
    rationale: "Query response time requirement enables real-time analytics capability"
}

trace "STK-002" satisfies "SYS-PERF-004" {
    rationale: "Streaming latency requirement supports real-time analytics for streaming metrics"
}

trace "STK-003" satisfies "SYS-MIG-003" {
    rationale: "Data validation framework ensures data availability and integrity during migration"
}

trace "STK-003" satisfies "SYS-MIG-004" {
    rationale: "Rollback procedures protect data availability in case of migration failures"
}

trace "STK-003" satisfies "SYS-REL-001" {
    rationale: "Automated backup/recovery ensures high availability during migration period"
}

trace "STK-003" satisfies "SYS-REL-002" {
    rationale: "Disaster recovery SLA directly satisfies availability requirement"
}

trace "STK-004" satisfies "SYS-PERF-003" {
    rationale: "Concurrent user support enables unified platform for diverse data teams"
}

trace "STK-004" satisfies "SYS-INT-001" {
    rationale: "REST API enables unified access for engineering and analytics workloads"
}

trace "STK-005" satisfies "SYS-MIG-001" {
    rationale: "Bidirectional sync enables zero downtime migration with seamless failback"
}

trace "STK-006" satisfies "SYS-SCALE-001" {
    rationale: "Auto-scaling compute supports 10x data and user growth"
}

trace "STK-006" satisfies "SYS-SCALE-002" {
    rationale: "Elastic storage directly addresses 100TB to 1PB scalability requirement"
}

trace "STK-007" satisfies "SYS-GOV-001" {
    rationale: "RBAC provides access control for compliance with GDPR/CCPA"
}

trace "STK-007" satisfies "SYS-GOV-002" {
    rationale: "PII detection supports compliance team's data protection requirements"
}

trace "STK-007" satisfies "SYS-GOV-005" {
    rationale: "Complete data lineage enables audit trails required by compliance team"
}

trace "STK-008" satisfies "SYS-MIG-002" {
    rationale: "Automated schema mapping accelerates migration timeline"
}

trace "STK-008" satisfies "SYS-MIG-005" {
    rationale: "Wave-based incremental migration fits 12-month timeline with Q1-Q4 milestones"
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY - SYSTEM REQUIREMENTS TO COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

trace "SYS-MIG-001" implements "LA-MIG-004" {
    rationale: "Conflict Resolver component implements bidirectional sync with conflict detection"
}

trace "SYS-MIG-002" implements "LA-MIG-002" {
    rationale: "Schema Converter component automates schema mapping from Oracle/Snowflake to Delta Lake"
}

trace "SYS-MIG-003" implements "LA-MIG-003" {
    rationale: "Data Validator component performs row count, checksum, and sampling validation"
}

trace "SYS-MIG-004" implements "LA-MIG-001" {
    rationale: "ETL Orchestrator includes failure recovery and rollback functionality"
}

trace "SYS-MIG-005" implements "LA-MIG-001" {
    rationale: "ETL Orchestrator manages wave-based migration with parallel execution"
}

trace "SYS-PERF-001" implements "LA-ANLZ-001" {
    rationale: "SQL Analytics Engine with Photon provides <5s query response time"
}

trace "SYS-PERF-002" implements "LA-TGT-003" {
    rationale: "Delta Lake Storage efficiently handles 100TB+ datasets with columnar format"
}

trace "SYS-PERF-003" implements "LA-TGT-001" {
    rationale: "Databricks Lakehouse Platform supports 500+ concurrent users with workload isolation"
}

trace "SYS-PERF-004" implements "LA-PROC-002" {
    rationale: "Streaming Pipeline Engine delivers <60s end-to-end latency with structured streaming"
}

trace "SYS-SCALE-001" implements "LA-TGT-001" {
    rationale: "Databricks Platform provides auto-scaling compute from 2-200 nodes"
}

trace "SYS-SCALE-002" implements "LA-TGT-003" {
    rationale: "Delta Lake on S3 provides elastic storage scaling to 1PB+"
}

trace "SYS-SCALE-003" implements "PA-CLOUD-001" {
    rationale: "AWS Cloud Infrastructure supports multi-region deployment"
}

trace "SYS-GOV-001" implements "LA-GOV-001" {
    rationale: "Access Control Manager enforces granular RBAC with dynamic data masking"
}

trace "SYS-GOV-002" implements "LA-GOV-001" {
    rationale: "Access Control Manager includes ML-based PII detection and classification"
}

trace "SYS-GOV-003" implements "LA-TGT-002" {
    rationale: "Unity Catalog provides multi-level data classification and tagging"
}

trace "SYS-GOV-004" implements "LA-GOV-003" {
    rationale: "Audit Logger enforces retention policies with automated archival"
}

trace "SYS-GOV-005" implements "LA-GOV-002" {
    rationale: "Lineage Tracker captures end-to-end column-level lineage automatically"
}

trace "SYS-INT-001" implements "LA-INT-001" {
    rationale: "API Gateway provides REST API with OAuth 2.0 authentication"
}

trace "SYS-INT-002" implements "LA-ANLZ-003" {
    rationale: "BI Connector Hub provides JDBC/ODBC connectivity for BI tools"
}

trace "SYS-INT-003" implements "LA-INT-003" {
    rationale: "Workflow Orchestrator integrates with CI/CD pipelines for automation"
}

trace "SYS-REL-001" implements "LA-TGT-003" {
    rationale: "Delta Lake provides automated incremental backups with time travel"
}

trace "SYS-REL-002" implements "PA-CLOUD-001" {
    rationale: "AWS multi-AZ deployment supports RPO <1hr and RTO <4hr"
}

trace "SYS-MON-001" implements "LA-MON-001" {
    rationale: "Observability Platform provides real-time monitoring with alerting"
}

trace "SYS-MON-002" implements "LA-PROC-003" {
    rationale: "Data Quality Engine tracks quality metrics with anomaly detection"
}

trace "SYS-MON-003" implements "LA-MON-002" {
    rationale: "Cost Optimizer tracks costs by workload with optimization recommendations"
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY - COMPONENTS TO PHYSICAL NODES
// ═══════════════════════════════════════════════════════════════════════════

trace "LA-TGT-001" deploys "PA-DBX-002" {
    rationale: "Databricks Lakehouse Platform deployed on customer data plane compute clusters"
}

trace "LA-TGT-002" deploys "PA-CATALOG-001" {
    rationale: "Unity Catalog deployed in regional metastore with centralized metadata"
}

trace "LA-TGT-003" deploys "PA-STORAGE-001" {
    rationale: "Delta Lake tables stored in S3 with intelligent tiering"
}

trace "LA-PROC-001" deploys "PA-DBX-002" {
    rationale: "Batch pipelines execute on Spark clusters in data plane"
}

trace "LA-PROC-002" deploys "PA-DBX-002" {
    rationale: "Streaming pipelines execute on dedicated streaming clusters"
}

trace "LA-GOV-001" deploys "PA-SEC-001" {
    rationale: "Access control enforced through AWS IAM and Unity Catalog integration"
}

trace "LA-INT-001" deploys "PA-NET-001" {
    rationale: "API Gateway deployed in VPC with security groups and network policies"
}

trace "LA-MON-001" deploys "PA-DBX-001" {
    rationale: "Observability platform uses Databricks system tables in control plane"
}

// ═══════════════════════════════════════════════════════════════════════════
// OPERATIONAL SCENARIOS
// ═══════════════════════════════════════════════════════════════════════════

// Note: Operational scenarios are typically documented separately but key
// scenarios are embedded in component descriptions and test cases would be:
//
// 1. INITIAL MIGRATION WAVE (Historical Data)
//    - ETL Orchestrator schedules Wave 1: 50 priority tables
//    - Schema Converter maps Oracle DDL to Delta Lake
//    - Migration Engine extracts 10 tables in parallel via JDBC
//    - Data Validator performs row count + checksum validation
//    - Success: 10TB migrated in 8 hours with 99.99% accuracy
//
// 2. INCREMENTAL SYNC (Transition Period)
//    - Conflict Resolver monitors CDC streams from Oracle
//    - Bidirectional sync every 5 minutes with <5min latency
//    - Data Quality Engine validates all incremental batches
//    - Automated alerts on validation failures
//
// 3. CUTOVER AND DECOMMISSIONING
//    - Final sync executed during maintenance window
//    - Applications switched to Databricks JDBC endpoints
//    - Legacy Oracle marked read-only for 30-day retention
//    - Verification: All queries return identical results
//
// 4. ONGOING PIPELINE OPERATIONS
//    - Batch pipelines run nightly (bronze→silver→gold)
//    - Streaming pipelines process 1M events/sec with <60s latency
//    - Auto-scaling adjusts clusters based on workload
//    - Data Quality Engine quarantines invalid records
//
// 5. DATA QUALITY INCIDENT RESPONSE
//    - Anomaly detected in silver layer (missing 10K rows)
//    - Alert Manager notifies data engineering team via Slack
//    - Lineage Tracker identifies root cause in bronze ingestion
//    - Rollback to previous Delta Lake version via time travel
//    - Re-run pipeline from checkpoint
//
// 6. DISASTER RECOVERY EXECUTION
//    - Simulated region failure in us-east-1
//    - Automated failover to us-west-2 within 3 minutes
//    - Delta Lake tables replicated across regions
//    - Applications reconnect to DR endpoint automatically
//    - Verification: RPO <1hr (15min actual), RTO <4hr (3min actual)

// ═══════════════════════════════════════════════════════════════════════════
// NON-FUNCTIONAL REQUIREMENTS & CONSTRAINTS
// ═══════════════════════════════════════════════════════════════════════════

// NON-FUNCTIONAL REQUIREMENTS are embedded in system requirements:
// - SYS-PERF-* addresses Performance
// - SYS-SCALE-* addresses Scalability
// - SYS-GOV-* addresses Security & Compliance
// - SYS-REL-* addresses Reliability
// - SYS-MON-* addresses Maintainability

// CONSTRAINTS are documented in metadata:
// - Budget: $3M total ($2M infrastructure, $1M services)
// - Timeline: 12 months (Q1-Q4 phased approach)
// - Team: 2 architects, 5 data engineers, 2 analysts
// - Technology Stack: Databricks, Delta Lake, Unity Catalog (AWS)
// - Maintain existing Oracle/Snowflake operations during migration
// - Zero planned downtime requirement
// - GDPR/CCPA/SOC2 compliance mandatory

// ═══════════════════════════════════════════════════════════════════════════
// END OF MODEL
// ═══════════════════════════════════════════════════════════════════════════
