// Databricks Migration - Oracle to Databricks
// Working MBSE Model for Arcadia 7 Dimensions Visualization

operational_analysis "Databricks Migration Operations" {
    actor "Data Engineer" {
        id: "ENG-001"
        description: "Manages ETL pipelines"
    }
    
    actor "Data Analyst" {
        id: "ANA-001"
        description: "Analyzes migrated data"
    }
    
    actor "DBA" {
        id: "DBA-001"
        description: "Database administrator"
    }
    
    actor "Business User" {
        id: "BUS-001"
        description: "Consumes reports"
    }
    
    operational_activity "Extract Oracle Data" {
        id: "OA-001"
        performed_by: "ENG-001"
        description: "Extract data from Oracle"
    }
    
    operational_activity "Transform to Delta" {
        id: "OA-002"
        performed_by: "ENG-001"
        description: "Transform to Delta Lake format"
    }
    
    operational_activity "Validate Migration" {
        id: "OA-003"
        performed_by: "DBA-001"
        description: "Validate data integrity"
    }
    
    operational_activity "Generate Reports" {
        id: "OA-004"
        performed_by: "ANA-001"
        description: "Create analytics reports"
    }
    
    operational_capability "Zero Downtime Migration" {
        id: "CAP-001"
        description: "Migrate without service interruption"
    }
    
    operational_capability "Data Quality Assurance" {
        id: "CAP-002"
        description: "Ensure data integrity and quality"
    }
}

system_analysis "Migration Platform" {
    requirement "REQ-MIGRATE-001" {
        description: "System shall migrate all Oracle tables to Delta format"
        priority: Critical
        verification_method: "Test"
    }
    
    requirement "REQ-PERF-001" {
        description: "System shall process 1TB per hour"
        priority: High
        verification_method: "Analysis"
    }
    
    requirement "REQ-SEC-001" {
        description: "System shall encrypt data in transit and at rest"
        priority: Critical
        verification_method: "Inspection"
    }
    
    requirement "REQ-AVAIL-001" {
        description: "System shall maintain 99.9 percent uptime"
        priority: High
        verification_method: "Test"
    }
    
    requirement "REQ-COMPAT-001" {
        description: "System shall support Oracle 11g and 12c"
        priority: High
        verification_method: "Test"
    }
    
    system_function "Extract Data" {
        id: "SF-EXTRACT"
        description: "Extract data from Oracle databases"
    }
    
    system_function "Transform Data" {
        id: "SF-TRANSFORM"
        description: "Transform to Delta Lake format"
    }
    
    system_function "Load Data" {
        id: "SF-LOAD"
        description: "Load into Databricks tables"
    }
    
    system_function "Validate Quality" {
        id: "SF-VALIDATE"
        description: "Validate data integrity"
    }
    
    system_function "Monitor Health" {
        id: "SF-MONITOR"
        description: "Monitor migration progress"
    }
    
    system_component "Oracle Connector" {
        id: "SC-CONNECTOR"
        description: "Oracle database connector"
    }
    
    system_component "ETL Engine" {
        id: "SC-ETL"
        description: "Data transformation engine"
    }
    
    system_component "Validator" {
        id: "SC-VALIDATOR"
        description: "Data quality validator"
    }
}

logical_architecture "Migration Solution" {
    component "OracleConnector" {
        name: "Oracle Connector"
        type: "Service"
        description: "Extracts data from Oracle"
        
        function "Connect"
        function "Extract"
        function "Stream"
    }
    
    component "TransformEngine" {
        name: "Transform Engine"
        type: "Process"
        description: "Transforms to Delta format"
        
        function "Parse"
        function "Transform"
        function "Optimize"
    }
    
    component "DeltaStorage" {
        name: "Delta Lake"
        type: "DataStore"
        description: "Delta Lake storage"
        
        function "Store"
        function "Version"
        function "Query"
    }
    
    component "DataValidator" {
        name: "Validator"
        type: "Service"
        description: "Validates data quality"
        
        function "Compare"
        function "Validate"
        function "Report"
    }
    
    component "Orchestrator" {
        name: "Workflow Orchestrator"
        type: "Service"
        description: "Orchestrates migration"
        
        function "Schedule"
        function "Monitor"
        function "Recover"
    }
    
    component "Dashboard" {
        name: "Monitoring Dashboard"
        type: "UI"
        description: "Migration metrics UI"
        
        function "Display"
        function "Alert"
        function "Export"
    }
    
    connection "OracleConnector" -> "TransformEngine" {
        label: "raw_data"
    }
    
    connection "TransformEngine" -> "DeltaStorage" {
        label: "delta_tables"
    }
    
    connection "DeltaStorage" -> "DataValidator" {
        label: "validation_data"
    }
    
    connection "Orchestrator" -> "OracleConnector" {
        label: "control"
    }
    
    connection "Orchestrator" -> "TransformEngine" {
        label: "control"
    }
    
    connection "DataValidator" -> "Dashboard" {
        label: "metrics"
    }
}

physical_architecture "AWS Deployment" {
    node "OracleDB" {
        id: "HW-ORACLE"
        type: hardware
        description: "Oracle database server"
    }
    
    node "EC2Connector" {
        id: "HW-EC2"
        type: hardware
        description: "EC2 connector instance"
    }
    
    node "DatabricksCluster" {
        id: "HW-DATABRICKS"
        type: hardware
        description: "Databricks cluster"
    }
    
    node "S3Bucket" {
        id: "HW-S3"
        type: hardware
        description: "S3 Delta Lake storage"
    }
    
    node "KafkaBroker" {
        id: "HW-KAFKA"
        type: hardware
        description: "Kafka streaming broker"
    }
    
    link "HW-ORACLE" -> "HW-EC2" {
        type: network
        description: "VPN tunnel"
    }
    
    link "HW-EC2" -> "HW-KAFKA" {
        type: network
        description: "AWS network"
    }
    
    link "HW-KAFKA" -> "HW-DATABRICKS" {
        type: network
        description: "Kafka stream"
    }
    
    link "HW-DATABRICKS" -> "HW-S3" {
        type: network
        description: "Storage link"
    }
    
    deployment "OracleConnector" -> "HW-EC2"
    deployment "TransformEngine" -> "HW-DATABRICKS"
    deployment "DeltaStorage" -> "HW-S3"
    deployment "Orchestrator" -> "HW-DATABRICKS"
}
