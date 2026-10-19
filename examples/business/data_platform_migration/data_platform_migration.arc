// ═══════════════════════════════════════════════════════════════════════════
// ENTERPRISE DATA PLATFORM MIGRATION
// ═══════════════════════════════════════════════════════════════════════════
// Domain:   Media and video streaming data platform
// Purpose:  Migrate the data estate from Oracle and Snowflake to a Databricks
//           lakehouse with no downtime, under GDPR and CCPA governance
// Budget:   3 M$ (2 M$ infrastructure, 1 M$ services), 12 months
// Team:     2 architects, 5 data engineers, 2 analysts
//
// Arcadia applied to an IT programme: the operational need of the data
// teams, the functions of the platform, the logical components grouped by
// layer, the cloud deployment, the acceptance tests on every requirement.
// No integrity levels: this is not a safety system, and the production gate
// checks requirements coverage, ICD completeness and methodology.
//
//   arclang check examples/business/data_platform_migration/data_platform_migration.arc
//   arclang gate  examples/business/data_platform_migration/data_platform_migration.arc
// ═══════════════════════════════════════════════════════════════════════════

model DataPlatformMigration {
    metadata {
        version: "2.0.0"
        description: "Enterprise data platform migration from Oracle/Snowflake to Databricks"
        domain: "media_streaming"
        project_phase: "architecture_design"
        budget_total_usd: 3000000
        timeline_months: 12
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STAKEHOLDER REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

requirements stakeholder {
    req "STK-001" "Total Cost of Ownership Reduction" {
        description: "The business must cut the total cost of ownership of the data platform by 40 percent over 3 years through consolidation and automation"
        priority: "Critical"
    }
    req "STK-002" "Real-Time and Batch Analytics" {
        description: "The platform must serve both streaming analytics and batch analytics on the video streaming metrics"
        priority: "Critical"
    }
    req "STK-003" "High Availability During Migration" {
        description: "Data must stay 99.9 percent available during the migration, with no data loss"
        priority: "Critical"
    }
    req "STK-004" "Unified Platform for Data Teams" {
        description: "Data engineering, data science and analytics must work on one platform"
        priority: "High"
    }
    req "STK-005" "Zero Downtime Migration" {
        description: "Users must see no planned downtime during the migration"
        priority: "Critical"
    }
    req "STK-006" "Scalable Architecture for Growth" {
        description: "The architecture must carry a tenfold data growth, from 100 TB to 1 PB, over 3 years"
        priority: "High"
    }
    req "STK-007" "Full Data Lineage and Audit Trails" {
        description: "Compliance needs complete data lineage and audit trails for GDPR and CCPA"
        priority: "Critical"
    }
    req "STK-008" "12-Month Migration Timeline" {
        description: "The migration must complete within 12 months with quarterly milestones"
        priority: "Critical"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM REQUIREMENTS
// ═══════════════════════════════════════════════════════════════════════════

requirements system {
    // Migration
    req "SYS-MIG-001" "Bidirectional Data Sync" {
        description: "The platform shall synchronise Oracle/Snowflake and Databricks in both directions with conflict resolution during the transition"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-MIG-002" "Automated Schema Mapping" {
        description: "The platform shall discover the Oracle/Snowflake schemas and map them to Delta Lake automatically"
        priority: "High"
    }
    req "SYS-MIG-003" "Data Validation Framework" {
        description: "Every migration batch shall be validated by row count reconciliation and checksum comparison"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-MIG-004" "Rollback Procedures" {
        description: "A migration wave shall be reversible to the previous stable state within 15 minutes"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-MIG-005" "Incremental Migration Waves" {
        description: "The migration shall proceed in waves at table granularity, with parallel execution"
        priority: "High"
    }
    // Performance
    req "SYS-PERF-001" "Query Response Time" {
        description: "The 95th percentile of the analytical queries shall answer in under 5 s"
        priority: "Critical"
    }
    req "SYS-PERF-002" "Large Dataset Support" {
        description: "Datasets above 100 TB shall be served with columnar storage and data skipping"
        priority: "High"
    }
    req "SYS-PERF-003" "Concurrent User Support" {
        description: "500 concurrent users shall be served with workload isolation and auto-scaling"
        priority: "High"
    }
    req "SYS-PERF-004" "Streaming Latency" {
        description: "99 percent of the streaming events shall be queryable within 60 s of their emission"
        priority: "Critical"
    }
    // Scalability and governance
    req "SYS-SCALE-001" "Auto-Scaling Compute" {
        description: "Compute clusters shall scale from 2 to 200 nodes with the workload"
        priority: "High"
    }
    req "SYS-SCALE-002" "Elastic Storage" {
        description: "Storage shall grow from 100 TB to over 1 PB with automatic tiering"
        priority: "High"
    }
    req "SYS-GOV-001" "Role-Based Access Control" {
        description: "Access shall be controlled at table, column and row level, with dynamic masking of personal data"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-GOV-002" "PII Detection and Classification" {
        description: "Personal data shall be detected and classified automatically"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-GOV-003" "Complete Data Lineage" {
        description: "Lineage shall be captured end to end at column level and be browsable"
        priority: "Critical"
        criticality: "High"
    }
    // Reliability and monitoring
    req "SYS-REL-001" "Automated Backup and Recovery" {
        description: "Incremental backups shall run every 6 hours with point-in-time recovery"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-REL-002" "Disaster Recovery SLA" {
        description: "Disaster recovery shall meet an RPO under 1 hour and an RTO under 4 hours"
        priority: "Critical"
        criticality: "High"
    }
    req "SYS-MON-001" "Real-Time Pipeline Monitoring" {
        description: "Pipelines shall be monitored in real time, with alerts on failures and SLA breaches"
        priority: "High"
    }
    req "SYS-MON-002" "Data Quality Metrics" {
        description: "Data quality metrics shall be tracked, with anomaly detection and root cause analysis"
        priority: "High"
    }
    req "SYS-MON-003" "Cost Tracking and Optimization" {
        description: "Cloud costs shall be tracked by workload and team, with optimisation recommendations"
        priority: "Medium"
    }
}

// Refinement: the system requirements refine the stakeholder needs
trace "SYS-MIG-005" refines "STK-001" { rationale: "Phased decommissioning of the legacy platforms cuts the TCO" }
trace "SYS-MON-003" refines "STK-001"
trace "SYS-PERF-001" refines "STK-002"
trace "SYS-PERF-004" refines "STK-002"
trace "SYS-MIG-003" refines "STK-003"
trace "SYS-REL-001" refines "STK-003"
trace "SYS-REL-002" refines "STK-003"
trace "SYS-PERF-003" refines "STK-004"
trace "SYS-MIG-001" refines "STK-005"
trace "SYS-MIG-004" refines "STK-005"
trace "SYS-SCALE-001" refines "STK-006"
trace "SYS-SCALE-002" refines "STK-006"
trace "SYS-PERF-002" refines "STK-006"
trace "SYS-GOV-001" refines "STK-007"
trace "SYS-GOV-002" refines "STK-007"
trace "SYS-GOV-003" refines "STK-007"
trace "SYS-MIG-002" refines "STK-008"
trace "SYS-MON-001" refines "STK-003"
trace "SYS-MON-002" refines "STK-003"

// ═══════════════════════════════════════════════════════════════════════════
// OPERATIONAL ANALYSIS: the data teams and what they do
// ═══════════════════════════════════════════════════════════════════════════

operational_analysis "Data Platform Operations" {
    actor "Data Analyst" {
        id: "OA-ACT-001"
        description: "Queries the business data and builds the reports and dashboards"
    }
    actor "Data Engineer" {
        id: "OA-ACT-002"
        description: "Builds and runs the ingestion and transformation pipelines"
    }
    actor "Platform Administrator" {
        id: "OA-ACT-003"
        description: "Runs the platform: capacity, backups, access, cost"
    }
    actor "Business User" {
        id: "OA-ACT-004"
        description: "Reads the reports and takes the decisions"
    }
    actor "Compliance Officer" {
        id: "OA-ACT-005"
        description: "Answers the GDPR and CCPA obligations: lineage, audit, retention"
    }
    entity "Streaming Data Estate" {
        id: "OA-ESTATE"
        description: "The data of the video streaming business, today on Oracle and Snowflake"
        activity "Query the historical data" { id: "OA-ACTIVITY-101" }
        activity "Transform and curate the data" { id: "OA-ACTIVITY-102" }
        activity "Migrate the data without downtime" { id: "OA-ACTIVITY-105" }
        activity "Monitor the platform health" { id: "OA-ACTIVITY-103" }
        activity "Generate the business reports" { id: "OA-ACTIVITY-104" }
        activity "Prove the lineage of a figure" { id: "OA-ACTIVITY-106" }
    }
    operational_capability "Data Migration" {
        id: "OA-CAP-001"
        description: "Move the estate from Oracle and Snowflake to Databricks with zero downtime"
        involves: ["OA-ACT-002", "OA-ACT-003", "OA-ESTATE"]
    }
    operational_capability "Analytics and Reporting" {
        id: "OA-CAP-002"
        description: "Serve real-time and batch analytics to the analysts and the business"
        involves: ["OA-ACT-001", "OA-ACT-004", "OA-ESTATE"]
    }
    operational_capability "Compliant Data Governance" {
        id: "OA-CAP-003"
        description: "Lineage, access control and audit trail on every personal data"
        involves: ["OA-ACT-005", "OA-ESTATE"]
    }
    interaction "Analytical queries" { from: "OA-ACT-001" to: "OA-ACTIVITY-101" }
    interaction "Pipeline changes" { from: "OA-ACT-002" to: "OA-ACTIVITY-102" }
    interaction "Migration waves" { from: "OA-ACT-003" to: "OA-ACTIVITY-105" }
    interaction "Reports" { from: "OA-ACTIVITY-104" to: "OA-ACT-004" }
    interaction "Lineage evidence" { from: "OA-ACTIVITY-106" to: "OA-ACT-005" }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM ANALYSIS: what the platform does
// ═══════════════════════════════════════════════════════════════════════════

system_analysis "Data Platform" {
    actor "Data Analyst" { id: "SA-ANALYST" }
    actor "Data Engineer" { id: "SA-ENGINEER" }
    actor "Platform Administrator" { id: "SA-ADMIN" }
    system_component "Lakehouse Data Platform" {
        id: "SC-DP"
        description: "The system under design: the migrated, governed, monitored data platform"
    }
    capability "Migrate a wave of tables" {
        id: "CAP-001"
        realizes: "OA-CAP-001"
        involves: ["SF-EXTRACT", "SF-SCHEMA", "SF-TRANSFORM", "SF-LOAD", "SF-VALIDATE"]
    }
    capability "Serve analytics" {
        id: "CAP-002"
        realizes: "OA-CAP-002"
        involves: ["SF-STREAM", "SF-QUERY", "SF-ML"]
    }
    capability "Govern the data" {
        id: "CAP-003"
        realizes: "OA-CAP-003"
        involves: ["SF-ACCESS", "SF-LINEAGE", "SF-AUDIT"]
    }

    function "Extract the source data" { id: "SF-EXTRACT" latency: 60 min description: "Read the Oracle and Snowflake tables of the wave" port out "raw" }
    function "Convert the schemas" { id: "SF-SCHEMA" latency: 10 min port in "raw" port out "mapped" }
    function "Transform to Delta" { id: "SF-TRANSFORM" latency: 90 min port in "mapped" port out "delta" }
    function "Load into the lakehouse" { id: "SF-LOAD" latency: 30 min port in "delta" port out "loaded" }
    function "Validate the migrated data" { id: "SF-VALIDATE" latency: 20 min description: "Row counts and checksums against the source" port in "loaded" port out "validated" }
    function "Reconcile and resolve conflicts" { id: "SF-SYNC" description: "Bidirectional sync while both platforms are live" }
    function "Roll a wave back" { id: "SF-ROLLBACK" latency: 15 min }
    function "Process the streaming events" { id: "SF-STREAM" latency: 60 s }
    function "Run the batch pipelines" { id: "SF-BATCH" description: "Bronze, silver and gold layers" }
    function "Enforce data quality" { id: "SF-DQ" }
    function "Serve the analytical queries" { id: "SF-QUERY" latency: 5 s }
    function "Train and serve models" { id: "SF-ML" }
    function "Control the access" { id: "SF-ACCESS" description: "RBAC, row and column level, masking of personal data" }
    function "Track the lineage" { id: "SF-LINEAGE" }
    function "Keep the audit trail" { id: "SF-AUDIT" }
    function "Monitor the pipelines" { id: "SF-MONITOR" }
    function "Optimise the cost" { id: "SF-COST" }
    function "Back up and recover" { id: "SF-BACKUP" }

    functional_exchange "raw rows" { id: "FE-001" from: "SF-EXTRACT.raw" to: "SF-SCHEMA.raw" }
    functional_exchange "mapped rows" { id: "FE-002" from: "SF-SCHEMA.mapped" to: "SF-TRANSFORM.mapped" }
    functional_exchange "delta files" { id: "FE-003" from: "SF-TRANSFORM.delta" to: "SF-LOAD.delta" }
    functional_exchange "loaded tables" { id: "FE-004" from: "SF-LOAD.loaded" to: "SF-VALIDATE.loaded" }

    functional_chain "Migration wave" {
        id: "FC-001"
        description: "One wave of tables, from extraction to validation, within the nightly window"
        involves: ["SF-EXTRACT", "SF-SCHEMA", "SF-TRANSFORM", "SF-LOAD", "SF-VALIDATE"]
        latency_budget: 4 h
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGICAL ARCHITECTURE: the components, grouped by layer
// ═══════════════════════════════════════════════════════════════════════════

architecture logical {
    // ---- Source data layer -------------------------------------------------
    component "Oracle Database Subsystem" {
        id: "LA-SRC-001"
        layer: "Source"
        description: "Legacy Oracle database: customer data, transactions, operational metrics (50 TB)"
        port out "oracle_rows"
        port in "oracle_sync"
        function "Expose the Oracle tables" { id: "LF-SRC-ORA" realizes: "SF-EXTRACT" }
    }
    component "Snowflake Warehouse Subsystem" {
        id: "LA-SRC-002"
        layer: "Source"
        description: "Existing Snowflake warehouse: analytics workloads and aggregated metrics (50 TB)"
        port out "snowflake_rows"
        port in "snowflake_sync"
        function "Expose the Snowflake tables" { id: "LF-SRC-SNOW" realizes: "SF-EXTRACT" }
    }
    // ---- Migration engine --------------------------------------------------
    component "ETL Orchestrator" {
        id: "LA-MIG-001"
        layer: "Migration"
        description: "Runs the migration waves, in parallel, with rollback"
        port in "oracle_rows"
        port in "snowflake_rows"
        port out "wave_rows"
        port out "delta_write"
        function "Schedule the waves" { id: "LF-WAVES" realizes: "SF-EXTRACT" }
        function "Load the Delta tables" { id: "LF-LOAD" realizes: "SF-LOAD" }
        function "Roll a wave back" { id: "LF-ROLLBACK" realizes: "SF-ROLLBACK" }
    }
    component "Schema Converter" {
        id: "LA-MIG-002"
        layer: "Migration"
        description: "Maps the Oracle and Snowflake schemas to Delta Lake"
        port in "wave_rows"
        port out "mapped_rows"
        function "Discover the source schemas" { id: "LF-DISCOVER" realizes: "SF-SCHEMA" }
        function "Generate the Delta schemas" { id: "LF-GEN" realizes: "SF-SCHEMA" }
    }
    component "Data Validator" {
        id: "LA-MIG-003"
        layer: "Migration"
        description: "Row count reconciliation, checksum comparison and sampling after each batch"
        port in "mapped_rows"
        port out "validation_report"
        function "Reconcile the row counts" { id: "LF-COUNTS" realizes: "SF-VALIDATE" }
        function "Compare the checksums" { id: "LF-CHECKSUM" realizes: "SF-VALIDATE" }
    }
    component "Conflict Resolver" {
        id: "LA-MIG-004"
        layer: "Migration"
        description: "Detects and resolves the conflicts of the bidirectional sync"
        port in "validation_report"
        port out "oracle_sync"
        port out "snowflake_sync"
        function "Detect the conflicts" { id: "LF-DETECT" realizes: "SF-SYNC" }
        function "Apply the resolution strategy" { id: "LF-RESOLVE" realizes: "SF-SYNC" }
    }
    // ---- Target platform ---------------------------------------------------
    component "Databricks Lakehouse Platform" {
        id: "LA-TGT-001"
        layer: "Target"
        description: "Unified lakehouse: data lake and warehouse on Delta Lake"
        port in "notebooks"
        port out "telemetry"
        function "Run the clusters" { id: "LF-CLUSTERS" realizes: "SF-QUERY" }
        function "Scale the compute" { id: "LF-SCALE" }
    }
    component "Unity Catalog" {
        id: "LA-TGT-002"
        layer: "Target"
        description: "Governance of the data and AI assets: catalog, grants, lineage"
        port out "grants"
        port out "lineage_events"
        function "Catalogue the assets" { id: "LF-CATALOG" realizes: "SF-ACCESS" }
        function "Emit the lineage events" { id: "LF-EMIT" realizes: "SF-LINEAGE" }
    }
    component "Delta Lake Storage" {
        id: "LA-TGT-003"
        layer: "Target"
        description: "ACID storage with versioning, time travel and schema evolution"
        port in "delta_write"
        port in "stream_write"
        port in "quarantine"
        port out "batch_read"
        port out "sql_read"
        function "Store the tables" { id: "LF-STORE" realizes: "SF-LOAD" }
        function "Keep the versions" { id: "LF-VERSIONS" realizes: "SF-BACKUP" }
    }
    // ---- Data processing ---------------------------------------------------
    component "Batch Pipeline Engine" {
        id: "LA-PROC-001"
        layer: "Processing"
        description: "Large-scale batch workloads in a medallion architecture (bronze, silver, gold)"
        port in "batch_read"
        port out "curated"
        function "Run the medallion pipelines" { id: "LF-MEDALLION" realizes: "SF-BATCH" }
        function "Transform to Delta" { id: "LF-TRANSFORM" realizes: "SF-TRANSFORM" }
    }
    component "Streaming Pipeline Engine" {
        id: "LA-PROC-002"
        layer: "Processing"
        description: "Real-time events with exactly-once semantics and sub-minute latency"
        port out "stream_write"
        function "Process the event streams" { id: "LF-STREAM" realizes: "SF-STREAM" }
    }
    component "Data Quality Engine" {
        id: "LA-PROC-003"
        layer: "Processing"
        description: "Quality rules, anomaly detection, quarantine of invalid data"
        port in "curated"
        port out "quarantine"
        port out "quality_metrics"
        function "Enforce the quality rules" { id: "LF-RULES" realizes: "SF-DQ" }
        function "Detect the anomalies" { id: "LF-ANOMALY" realizes: "SF-DQ" }
    }
    // ---- Governance --------------------------------------------------------
    component "Access Control Manager" {
        id: "LA-GOV-001"
        layer: "Governance"
        description: "RBAC and ABAC with dynamic masking of personal data"
        port in "grants"
        port out "access_events"
        function "Enforce the grants" { id: "LF-GRANTS" realizes: "SF-ACCESS" }
        function "Classify the personal data" { id: "LF-PII" realizes: "SF-ACCESS" }
    }
    component "Lineage Tracker" {
        id: "LA-GOV-002"
        layer: "Governance"
        description: "End-to-end lineage from the sources to the dashboards"
        port in "lineage_events"
        function "Build the lineage graph" { id: "LF-GRAPH" realizes: "SF-LINEAGE" }
    }
    component "Audit Logger" {
        id: "LA-GOV-003"
        layer: "Governance"
        description: "Centralised audit log for GDPR, CCPA and SOC 2"
        port in "access_events"
        function "Record the audit trail" { id: "LF-AUDIT" realizes: "SF-AUDIT" }
    }
    // ---- Integration -------------------------------------------------------
    component "API Gateway" {
        id: "LA-INT-001"
        layer: "Integration"
        description: "REST API for data access, job submission and metadata queries"
        port out "job_requests"
        port out "metadata_queries"
        function "Serve the REST API" { id: "LF-API" realizes: "SF-QUERY" }
    }
    component "Metadata Manager" {
        id: "LA-INT-002"
        layer: "Integration"
        description: "Technical, business and operational metadata"
        port in "metadata_queries"
        function "Answer the metadata queries" { id: "LF-META" realizes: "SF-LINEAGE" }
    }
    component "Workflow Orchestrator" {
        id: "LA-INT-003"
        layer: "Integration"
        description: "Multi-step workflows across the pipelines and the model training"
        port in "job_requests"
        port out "notebooks"
        function "Run the workflows" { id: "LF-WORKFLOWS" realizes: "SF-BATCH" }
    }
    // ---- Analytics ---------------------------------------------------------
    component "SQL Analytics Engine" {
        id: "LA-ANLZ-001"
        layer: "Analytics"
        description: "SQL engine for the BI workloads, with result caching"
        port in "sql_read"
        port out "results"
        function "Execute the SQL queries" { id: "LF-SQL" realizes: "SF-QUERY" }
    }
    component "ML Workspace" {
        id: "LA-ANLZ-002"
        layer: "Analytics"
        description: "Notebooks, MLflow tracking and model serving"
        port in "results"
        function "Train and serve the models" { id: "LF-ML" realizes: "SF-ML" }
    }
    component "BI Connector Hub" {
        id: "LA-ANLZ-003"
        layer: "Analytics"
        description: "Semantic layer and connectors for Tableau, Power BI and Looker"
        port in "results"
        function "Serve the BI tools" { id: "LF-BI" realizes: "SF-QUERY" }
    }
    // ---- Monitoring and operations -----------------------------------------
    component "Observability Platform" {
        id: "LA-MON-001"
        layer: "Monitoring"
        description: "Metrics, logs and traces of every component"
        port in "telemetry"
        port in "quality_metrics"
        port out "alerts"
        function "Collect the telemetry" { id: "LF-TELEMETRY" realizes: "SF-MONITOR" }
    }
    component "Cost Optimizer" {
        id: "LA-MON-002"
        layer: "Monitoring"
        description: "Cost by workload and team, with recommendations"
        port out "cost_alerts"
        function "Attribute and optimise the cost" { id: "LF-COST" realizes: "SF-COST" }
    }
    component "Alert Manager" {
        id: "LA-MON-003"
        layer: "Monitoring"
        description: "Alert routing and on-call management"
        port in "alerts"
        port in "cost_alerts"
        function "Route the alerts" { id: "LF-ALERTS" realizes: "SF-MONITOR" }
    }

    // ---- Exchanges: migration dataflow -------------------------------------
    component_exchange "Oracle rows" { from_port: "LA-SRC-001.oracle_rows" to_port: "LA-MIG-001.oracle_rows" protocol: "JDBC over Direct Connect" }
    component_exchange "Snowflake rows" { from_port: "LA-SRC-002.snowflake_rows" to_port: "LA-MIG-001.snowflake_rows" protocol: "Snowflake connector" }
    component_exchange "Wave rows" { from_port: "LA-MIG-001.wave_rows" to_port: "LA-MIG-002.wave_rows" }
    component_exchange "Mapped rows" { from_port: "LA-MIG-002.mapped_rows" to_port: "LA-MIG-003.mapped_rows" }
    component_exchange "Delta writes" { from_port: "LA-MIG-001.delta_write" to_port: "LA-TGT-003.delta_write" protocol: "Delta Lake API" }
    component_exchange "Validation report" { from_port: "LA-MIG-003.validation_report" to_port: "LA-MIG-004.validation_report" }
    component_exchange "Oracle sync" { from_port: "LA-MIG-004.oracle_sync" to_port: "LA-SRC-001.oracle_sync" protocol: "JDBC over Direct Connect" }
    component_exchange "Snowflake sync" { from_port: "LA-MIG-004.snowflake_sync" to_port: "LA-SRC-002.snowflake_sync" protocol: "Snowflake connector" }
    // ---- Exchanges: processing ---------------------------------------------
    component_exchange "Batch reads" { from_port: "LA-TGT-003.batch_read" to_port: "LA-PROC-001.batch_read" }
    component_exchange "Curated tables" { from_port: "LA-PROC-001.curated" to_port: "LA-PROC-003.curated" }
    component_exchange "Stream writes" { from_port: "LA-PROC-002.stream_write" to_port: "LA-TGT-003.stream_write" }
    component_exchange "Quarantined rows" { from_port: "LA-PROC-003.quarantine" to_port: "LA-TGT-003.quarantine" }
    // ---- Exchanges: governance ---------------------------------------------
    component_exchange "Grants" { from_port: "LA-TGT-002.grants" to_port: "LA-GOV-001.grants" }
    component_exchange "Lineage events" { from_port: "LA-TGT-002.lineage_events" to_port: "LA-GOV-002.lineage_events" }
    component_exchange "Access events" { from_port: "LA-GOV-001.access_events" to_port: "LA-GOV-003.access_events" }
    // ---- Exchanges: integration and analytics ------------------------------
    component_exchange "Job requests" { from_port: "LA-INT-001.job_requests" to_port: "LA-INT-003.job_requests" protocol: "REST" }
    component_exchange "Metadata queries" { from_port: "LA-INT-001.metadata_queries" to_port: "LA-INT-002.metadata_queries" protocol: "REST" }
    component_exchange "Notebook jobs" { from_port: "LA-INT-003.notebooks" to_port: "LA-TGT-001.notebooks" protocol: "Databricks Jobs API" }
    component_exchange "SQL reads" { from_port: "LA-TGT-003.sql_read" to_port: "LA-ANLZ-001.sql_read" }
    component_exchange "Query results to ML" { from_port: "LA-ANLZ-001.results" to_port: "LA-ANLZ-002.results" }
    component_exchange "Query results to BI" { from_port: "LA-ANLZ-001.results" to_port: "LA-ANLZ-003.results" protocol: "ODBC / JDBC" }
    // ---- Exchanges: monitoring ---------------------------------------------
    component_exchange "Platform telemetry" { from_port: "LA-TGT-001.telemetry" to_port: "LA-MON-001.telemetry" protocol: "OpenTelemetry" }
    component_exchange "Quality metrics" { from_port: "LA-PROC-003.quality_metrics" to_port: "LA-MON-001.quality_metrics" protocol: "OpenTelemetry" }
    component_exchange "Alerts" { from_port: "LA-MON-001.alerts" to_port: "LA-MON-003.alerts" }
    component_exchange "Cost alerts" { from_port: "LA-MON-002.cost_alerts" to_port: "LA-MON-003.cost_alerts" }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICAL ARCHITECTURE: the cloud deployment
// ═══════════════════════════════════════════════════════════════════════════

physical_architecture "Data Platform Physical Deployment" {
    node "On-Premises Data Center" {
        id: "PA-ONPREM-001"
        description: "Oracle estate and the Snowflake account reached through it"
        deploys "LA-SRC-001" { partition: "Oracle RAC" }
        deploys "LA-SRC-002" { partition: "Snowflake account (SaaS)" }
    }
    node "Migration Engine Node" {
        id: "PA-MIG-001"
        processor: "AWS EC2 c6g.8xlarge"
        ram: 64 GB
        description: "Dedicated 10 Gbps network to the sources"
        deploys "LA-MIG-001" { partition: "ETL Orchestrator" max_parallel_jobs: 100 }
        deploys "LA-MIG-002" { partition: "Schema Converter" }
        deploys "LA-MIG-003" { partition: "Data Validator" validation_threads: 32 }
        deploys "LA-MIG-004" { partition: "Conflict Resolver" resolution_strategy: "timestamp-based" }
    }
    node "AWS Cloud Infrastructure" {
        id: "PA-CLOUD-001"
        processor: "AWS Graviton3 (ARM64)"
        region: "us-east-1"
        availability_zones: "us-east-1a, us-east-1b, us-east-1c"
        storage: 100000 GB
        deploys "LA-TGT-001" { partition: "Databricks control plane" }
        deploys "LA-TGT-002" { partition: "Unity Catalog service" }
        deploys "LA-TGT-003" { partition: "Delta Lake on S3, tiered to Glacier" backup_frequency: 6 h }
    }
    node "Databricks Workspace Cluster" {
        id: "PA-DBX-002"
        processor: "Databricks managed (AWS i3.xlarge)"
        cluster_type: "Multi-node autoscaling, 2 to 200 workers"
        deploys "LA-PROC-001" { partition: "Batch processing" }
        deploys "LA-PROC-002" { partition: "Streaming processing" }
        deploys "LA-PROC-003" { partition: "Data quality engine" }
    }
    node "Governance Control Node" {
        id: "PA-GOV-001"
        processor: "AWS EC2 c6g.4xlarge"
        ram: 32 GB
        storage: 1000 GB
        deploys "LA-GOV-001" { partition: "Access control service" }
        deploys "LA-GOV-002" { partition: "Lineage tracking service" }
        deploys "LA-GOV-003" { partition: "Audit logging service" retention_period: "7 years" }
    }
    node "Integration Gateway Node" {
        id: "PA-INT-001"
        processor: "AWS EC2 m6g.2xlarge"
        ram: 32 GB
        load_balancer: "AWS ALB"
        deploys "LA-INT-001" { partition: "API gateway" max_connections: 10000 }
        deploys "LA-INT-002" { partition: "Metadata manager" }
        deploys "LA-INT-003" { partition: "Workflow orchestrator" }
    }
    node "Analytics Compute Node" {
        id: "PA-ANLZ-001"
        processor: "Databricks SQL Warehouse (serverless, 2X-Large)"
        auto_suspend: 10 min
        deploys "LA-ANLZ-001" { partition: "SQL analytics engine" max_concurrent_queries: 500 }
        deploys "LA-ANLZ-002" { partition: "ML workspace" gpu: "NVIDIA A100" }
        deploys "LA-ANLZ-003" { partition: "BI connector hub" }
    }
    node "Monitoring Infrastructure" {
        id: "PA-MON-001"
        processor: "AWS EC2 r6g.xlarge"
        ram: 32 GB
        storage: 5000 GB
        deploys "LA-MON-001" { partition: "Observability platform" metrics_retention: "90 days" }
        deploys "LA-MON-002" { partition: "Cost optimizer" }
        deploys "LA-MON-003" { partition: "Alert manager" notification_channels: "PagerDuty, Slack, email" }
    }

    // One link per pair of nodes that exchange data
    physical_link "Direct Connect to the sources" { id: "PL-001" from: "PA-ONPREM-001" to: "PA-MIG-001" protocol: "AWS Direct Connect, MACsec" bandwidth: 10 Gbps }
    physical_link "Migration to the lakehouse" { id: "PL-002" from: "PA-MIG-001" to: "PA-CLOUD-001" protocol: "HTTPS / TLS 1.3" bandwidth: 10 Gbps }
    physical_link "Lakehouse to the workspace cluster" { id: "PL-003" from: "PA-CLOUD-001" to: "PA-DBX-002" protocol: "AWS VPC, TLS 1.3" bandwidth: 100 Gbps }
    physical_link "Lakehouse to the governance node" { id: "PL-004" from: "PA-CLOUD-001" to: "PA-GOV-001" protocol: "AWS VPC, TLS 1.3" bandwidth: 10 Gbps }
    physical_link "Lakehouse to the analytics node" { id: "PL-005" from: "PA-CLOUD-001" to: "PA-ANLZ-001" protocol: "AWS VPC, TLS 1.3" bandwidth: 100 Gbps }
    physical_link "Integration to the lakehouse" { id: "PL-006" from: "PA-INT-001" to: "PA-CLOUD-001" protocol: "AWS VPC, HTTPS" bandwidth: 10 Gbps }
    physical_link "Lakehouse telemetry" { id: "PL-007" from: "PA-CLOUD-001" to: "PA-MON-001" protocol: "OpenTelemetry over gRPC" bandwidth: 1 Gbps }
    physical_link "Workspace telemetry" { id: "PL-008" from: "PA-DBX-002" to: "PA-MON-001" protocol: "OpenTelemetry over gRPC" bandwidth: 1 Gbps }
    physical_link "DR replication to us-west-2" { id: "PL-009" from: "PA-CLOUD-001" to: "PA-ONPREM-001" protocol: "S3 cross-region replication, then restore to the sources on rollback" bandwidth: 10 Gbps }
}

// ═══════════════════════════════════════════════════════════════════════════
// DATA MODEL
// ═══════════════════════════════════════════════════════════════════════════

class "ViewingEvent" {
    id: "CL-001"
    description: "One playback event of the streaming service"
    event_id: "string"
    customer_id: "string"
    content_id: "string"
    watched_seconds: "int"
    emitted_at: "timestamp"
}
class "CustomerRecord" {
    id: "CL-002"
    description: "Personal data under GDPR and CCPA"
    customer_id: "string"
    email: "string"
    country: "string"
    consent: "ConsentState"
}
enumeration "ConsentState" {
    id: "EN-001"
    values: ["Given", "Withdrawn", "Unknown"]
}
exchange_item "Viewing events" { id: "EI-001" mechanism: "FLOW" elements: ["CL-001"] }
exchange_item "Customer records" { id: "EI-002" mechanism: "DATA" elements: ["CL-002"] }

// ═══════════════════════════════════════════════════════════════════════════
// MODES AND SCENARIO
// ═══════════════════════════════════════════════════════════════════════════

state_machine "Migration wave states" {
    initial: "Planned"
    mode Planned { description: "Tables of the wave selected, source still the system of record" }
    mode Extracting { description: "Extraction and conversion running" }
    mode Validating { description: "Row counts and checksums compared" }
    mode DualRunning { description: "Both platforms live, bidirectional sync" }
    mode CutOver { description: "Lakehouse is the system of record" }
    state RolledBack { description: "Wave reverted to the previous stable state" }
    transition Planned -> Extracting { trigger: "LF-WAVES" }
    transition Extracting -> Validating { trigger: "LF-LOAD" }
    transition Validating -> DualRunning { trigger: "LF-CHECKSUM" guard: "counts and checksums match" }
    transition Validating -> RolledBack { trigger: "LF-ROLLBACK" guard: "validation failed" timing: 15 min }
    transition DualRunning -> CutOver { trigger: "LF-RESOLVE" guard: "no conflict for two weeks" }
    transition DualRunning -> RolledBack { trigger: "LF-ROLLBACK" guard: "unresolved conflicts" timing: 15 min }
}

scenario "Migration wave" {
    participants: ["LA-SRC-001", "LA-MIG-001", "LA-MIG-002", "LA-MIG-003", "LA-MIG-004", "LA-TGT-003"]
    message "LA-SRC-001" -> "LA-MIG-001" "Oracle rows"
    message "LA-MIG-001" -> "LA-MIG-002" "wave rows"
    message "LA-MIG-002" -> "LA-MIG-003" "mapped rows"
    message "LA-MIG-001" -> "LA-TGT-003" "Delta writes"
    message "LA-MIG-003" -> "LA-MIG-004" "validation report"
    fragment opt "conflicts found" {
        message "LA-MIG-004" -> "LA-SRC-001" "Oracle sync"
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ACCEPTANCE TESTS
// ═══════════════════════════════════════════════════════════════════════════

test_case "TC-001" { verifies: ["STK-001", "SYS-MON-003"] method: "analysis" description: "TCO model reviewed against the cost reports of the first two quarters" }
test_case "TC-002" { verifies: ["STK-002", "SYS-PERF-001", "SYS-PERF-004"] method: "test" description: "Query percentiles and streaming latency measured on the production workload replay" }
test_case "TC-003" { verifies: ["STK-003", "STK-005", "SYS-MIG-001", "SYS-MIG-003"] method: "test" description: "Dual-run of a wave: availability, reconciliation and conflict resolution observed" }
test_case "TC-004" { verifies: ["STK-004", "SYS-PERF-003"] method: "demonstration" description: "Engineering, science and analytics workloads of 500 users run on the one platform" }
test_case "TC-005" { verifies: ["STK-006", "SYS-SCALE-001", "SYS-SCALE-002", "SYS-PERF-002"] method: "test" description: "Load test at 1 PB synthetic data, cluster scaled to 200 nodes" }
test_case "TC-006" { verifies: ["STK-007", "SYS-GOV-001", "SYS-GOV-002", "SYS-GOV-003"] method: "inspection" description: "Compliance walkthrough: lineage of a dashboard figure, masking, access and audit records" }
test_case "TC-007" { verifies: ["STK-008", "SYS-MIG-002", "SYS-MIG-005"] method: "demonstration" description: "Quarterly milestone reviews: waves migrated against the plan" }
test_case "TC-008" { verifies: ["SYS-MIG-004"] method: "test" description: "Rollback of a wave timed under 15 minutes" }
test_case "TC-009" { verifies: ["SYS-REL-001", "SYS-REL-002"] method: "test" description: "Disaster recovery exercise: RPO and RTO measured" }
test_case "TC-010" { verifies: ["SYS-MON-001", "SYS-MON-002"] method: "demonstration" description: "Pipeline failure and quality anomaly injected, alerts received" }

constraint "Migration wave fits the nightly window" {
    id: "CST-001"
    assert: sum("FC-001", latency) <= "FC-001".latency_budget
}

// ═══════════════════════════════════════════════════════════════════════════
// TRACEABILITY
// ═══════════════════════════════════════════════════════════════════════════

// The platform satisfies the stakeholder needs
trace "SC-DP" satisfies "STK-001"
trace "SC-DP" satisfies "STK-002"
trace "SC-DP" satisfies "STK-003"
trace "SC-DP" satisfies "STK-004"
trace "SC-DP" satisfies "STK-005"
trace "SC-DP" satisfies "STK-006"
trace "SC-DP" satisfies "STK-007"
trace "SC-DP" satisfies "STK-008"

// The logical components satisfy the system requirements
trace "LA-SRC-001" satisfies "SYS-MIG-001" { rationale: "Oracle stays a sync endpoint during the dual run" }
trace "LA-SRC-002" satisfies "SYS-MIG-001" { rationale: "Snowflake stays a sync endpoint during the dual run" }
trace "LA-MIG-001" satisfies "SYS-MIG-004" { rationale: "Rollback of a wave to the previous Delta version" }
trace "LA-MIG-001" satisfies "SYS-MIG-005" { rationale: "Wave scheduling at table granularity, 100 parallel jobs" }
trace "LA-MIG-002" satisfies "SYS-MIG-002" { rationale: "Schema discovery and Delta schema generation" }
trace "LA-MIG-003" satisfies "SYS-MIG-003" { rationale: "Row counts and checksums after each batch" }
trace "LA-MIG-004" satisfies "SYS-MIG-001" { rationale: "Conflict detection and resolution of the bidirectional sync" }
trace "LA-TGT-001" satisfies "SYS-PERF-003" { rationale: "Workload isolation and auto-scaling for 500 users" }
trace "LA-TGT-001" satisfies "SYS-SCALE-001" { rationale: "Clusters from 2 to 200 nodes" }
trace "LA-TGT-002" satisfies "SYS-GOV-003" { rationale: "Lineage events of every query and job" }
trace "LA-TGT-003" satisfies "SYS-PERF-002" { rationale: "Columnar storage with data skipping" }
trace "LA-TGT-003" satisfies "SYS-SCALE-002" { rationale: "S3 with automatic tiering" }
trace "LA-TGT-003" satisfies "SYS-REL-001" { rationale: "Versions and incremental backups every 6 hours" }
trace "LA-TGT-003" satisfies "SYS-REL-002" { rationale: "Cross-region replication: RPO under 1 h, RTO under 4 h" }
trace "LA-PROC-001" satisfies "SYS-PERF-002" { rationale: "Medallion pipelines sized for the 100 TB datasets" }
trace "LA-PROC-002" satisfies "SYS-PERF-004" { rationale: "Sub-minute streaming latency" }
trace "LA-PROC-003" satisfies "SYS-MON-002" { rationale: "Quality metrics and anomaly detection" }
trace "LA-GOV-001" satisfies "SYS-GOV-001" { rationale: "RBAC, row and column level, dynamic masking" }
trace "LA-GOV-001" satisfies "SYS-GOV-002" { rationale: "Classification of the personal data" }
trace "LA-GOV-002" satisfies "SYS-GOV-003" { rationale: "Column-level lineage graph" }
trace "LA-GOV-003" satisfies "SYS-GOV-001" { rationale: "Audit trail of every access decision" }
trace "LA-INT-001" satisfies "SYS-PERF-003" { rationale: "Rate-limited API in front of the platform" }
trace "LA-INT-002" satisfies "SYS-GOV-003" { rationale: "Business metadata joined to the lineage" }
trace "LA-INT-003" satisfies "SYS-MIG-005" { rationale: "Wave workflows run in parallel" }
trace "LA-ANLZ-001" satisfies "SYS-PERF-001" { rationale: "Result caching for the BI queries" }
trace "LA-ANLZ-002" satisfies "SYS-PERF-003" { rationale: "Science workloads isolated from the BI warehouse" }
trace "LA-ANLZ-003" satisfies "SYS-PERF-001" { rationale: "Semantic layer answers the dashboards from the cache" }
trace "LA-MON-001" satisfies "SYS-MON-001" { rationale: "Real-time pipeline telemetry" }
trace "LA-MON-002" satisfies "SYS-MON-003" { rationale: "Cost attribution and recommendations" }
trace "LA-MON-003" satisfies "SYS-MON-001" { rationale: "Alerts routed to the on-call engineer" }

// System functions realize the operational activities
trace "SF-EXTRACT" realizes "OA-ACTIVITY-105"
trace "SF-TRANSFORM" realizes "OA-ACTIVITY-102"
trace "SF-QUERY" realizes "OA-ACTIVITY-101"
trace "SF-MONITOR" realizes "OA-ACTIVITY-103"
trace "SF-ML" realizes "OA-ACTIVITY-104"
trace "SF-LINEAGE" realizes "OA-ACTIVITY-106"

diagram "Migration engine" {
    view: lab
    description: "From the sources through the migration engine to Delta Lake"
    shows: ["LA-SRC-001", "LA-SRC-002", "LA-MIG-001", "LA-MIG-002", "LA-MIG-003", "LA-MIG-004", "LA-TGT-003"]
}
diagram "Cloud deployment" {
    view: pab
    shows: ["PA-CLOUD-001"]
    context: true
}
