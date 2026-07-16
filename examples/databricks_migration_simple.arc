// Databricks Migration Architecture - Oracle to Databricks
// Simplified MBSE Model following Arcadia 7 Dimensions

// ========================================
// 1. OPERATIONAL ANALYSIS
// ========================================

operational_analysis "Databricks Migration" {
    actor "Data Analyst" {
        id: "OA-ACT-001"
        description: "Analyzes business data and creates reports"
    }
    
    actor "Data Engineer" {
        id: "OA-ACT-002"
        description: "Manages data pipelines and ETL processes"
    }
    
    actor "Database Administrator" {
        id: "OA-ACT-003"
        description: "Manages database infrastructure"
    }
    
    actor "Business User" {
        id: "OA-ACT-004"
        description: "Consumes data insights and reports"
    }
    
    operational_activity "Query Historical Data" {
        id: "OA-ACTIVITY-101"
        performed_by: "OA-ACT-001"
        description: "Access and analyze historical business data"
    }
    
    operational_activity "Transform Data" {
        id: "OA-ACTIVITY-102"
        performed_by: "OA-ACT-002"
        description: "Clean transform and prepare data for analysis"
    }
    
    operational_activity "Monitor System Health" {
        id: "OA-ACTIVITY-103"
        performed_by: "OA-ACT-003"
        description: "Monitor database performance and availability"
    }
    
    operational_activity "Generate Business Reports" {
        id: "OA-ACTIVITY-104"
        performed_by: "OA-ACT-001"
        description: "Create dashboards and analytics reports"
    }
    
    operational_capability "Data Migration" {
        id: "OA-CAP-001"
        description: "Migrate data from Oracle to Databricks with zero downtime"
    }
    
    operational_capability "Analytics and Reporting" {
        id: "OA-CAP-002"
        description: "Provide real-time analytics and reporting capabilities"
    }
}

// ========================================
// 2. SYSTEM ANALYSIS
// ========================================

system_analysis "Migration System" {
    requirement "REQ-001" {
        description: "System shall migrate all Oracle tables to Databricks Delta format"
        priority: Critical
        verification_method: "Test"
    }
    
    requirement "REQ-002" {
        description: "System shall support zero-downtime migration"
        priority: Critical
        verification_method: "Test"
    }
    
    requirement "REQ-003" {
        description: "System shall process 1TB of data per hour"
        priority: High
        verification_method: "Analysis"
    }
    
    requirement "REQ-004" {
        description: "System shall maintain 99.9 percent uptime during migration"
        priority: High
        verification_method: "Test"
    }
    
    requirement "REQ-005" {
        description: "System shall encrypt all data in transit and at rest"
        priority: Critical
        verification_method: "Inspection"
    }
    
    system_function "Extract Data" {
        id: "SF-001"
        description: "Extract data from Oracle source databases"
    }
    
    system_function "Transform Data" {
        id: "SF-002"
        description: "Transform data to Delta Lake format"
    }
    
    system_function "Load Data" {
        id: "SF-003"
        description: "Load transformed data into Databricks"
    }
    
    system_function "Validate Data" {
        id: "SF-004"
        description: "Validate data integrity and quality"
    }
    
    system_function "Monitor Migration" {
        id: "SF-005"
        description: "Monitor migration progress and health"
    }
    
    system_component "Oracle Connector" {
        id: "SC-001"
        description: "Connects to Oracle and extracts data"
        implements: ["SF-001"]
    }
    
    system_component "ETL Engine" {
        id: "SC-002"
        description: "Transforms data to Delta format"
        implements: ["SF-002"]
    }
    
    system_component "Data Validator" {
        id: "SC-003"
        description: "Validates data quality"
        implements: ["SF-004"]
    }
}

// ========================================
// 3. LOGICAL ARCHITECTURE
// ========================================

logical_architecture "Migration Architecture" {
    component "Oracle Connector Service" {
        name: "Oracle Connector"
        type: "Service"
        description: "Connects to Oracle databases and extracts data"
        
        function "Connect to Oracle"
        function "Query Tables"
        function "Stream Data"
    }
    
    component "ETL Processing Engine" {
        name: "ETL Engine"
        type: "Process"
        description: "Transforms data from Oracle to Delta format"
        
        function "Parse Schema"
        function "Transform Records"
        function "Optimize Partitions"
        function "Write Delta"
    }
    
    component "Delta Lake Storage" {
        name: "Delta Storage"
        type: "DataStore"
        description: "Stores data in Delta Lake format on S3"
        
        function "Store Tables"
        function "Version Control"
        function "ACID Transactions"
    }
    
    component "Data Validation Service" {
        name: "Data Validator"
        type: "Service"
        description: "Validates data quality and integrity"
        
        function "Compare Schemas"
        function "Validate Counts"
        function "Check Data Quality"
    }
    
    component "Migration Orchestration Service" {
        name: "Migration Orchestrator"
        type: "Service"
        description: "Orchestrates migration workflow"
        
        function "Schedule Jobs"
        function "Monitor Progress"
        function "Handle Failures"
    }
    
    component "Monitoring Dashboard UI" {
        name: "Monitoring Dashboard"
        type: "UI"
        description: "Visualizes migration progress and metrics"
        
        function "Display Progress"
        function "Show Alerts"
        function "Generate Reports"
    }
    
    connection "Oracle Connector Service" -> "ETL Processing Engine" {
        label: "Raw data stream"
    }
    
    connection "ETL Processing Engine" -> "Delta Lake Storage" {
        label: "Delta tables"
    }
    
    connection "Delta Lake Storage" -> "Data Validation Service" {
        label: "Target data"
    }
    
    connection "Migration Orchestration Service" -> "Oracle Connector Service" {
        label: "Start extraction"
    }
    
    connection "Migration Orchestration Service" -> "ETL Processing Engine" {
        label: "Start transformation"
    }
    
    connection "Data Validation Service" -> "Monitoring Dashboard UI" {
        label: "Validation results"
    }
}

// ========================================
// 4. PHYSICAL ARCHITECTURE
// ========================================

physical_architecture "Cloud Deployment" {
    node "Oracle Database Server" {
        id: "PN-001"
        type: hardware
        description: "On-premise Oracle database"
    }
    
    node "AWS EC2 Connector Instance" {
        id: "PN-002"
        type: hardware
        description: "EC2 instance running Oracle connector"
    }
    
    node "Databricks Compute Cluster" {
        id: "PN-003"
        type: hardware
        description: "Databricks compute cluster"
    }
    
    node "S3 Storage Bucket" {
        id: "PN-004"
        type: hardware
        description: "S3 storage for Delta Lake"
    }
    
    node "Kafka Message Broker" {
        id: "PN-005"
        type: hardware
        description: "MSK Kafka cluster for data streaming"
    }
    
    link "PN-001" -> "PN-002" {
        type: network
        description: "Secure VPN connection"
    }
    
    link "PN-002" -> "PN-005" {
        type: network
        description: "AWS internal network"
    }
    
    link "PN-005" -> "PN-003" {
        type: network
        description: "Kafka to Databricks"
    }
    
    link "PN-003" -> "PN-004" {
        type: network
        description: "Databricks to S3"
    }
    
    deployment "Oracle Connector Service" -> "PN-002"
    deployment "ETL Processing Engine" -> "PN-003"
    deployment "Delta Lake Storage" -> "PN-004"
    deployment "Migration Orchestration Service" -> "PN-003"
}
