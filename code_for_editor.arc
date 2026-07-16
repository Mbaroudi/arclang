// Databricks Migration - Complete 7D Model

operational_analysis "Databricks Migration" {
    actor "Data Engineer" {
        id: "ENG-001"
        description: "Manages ETL pipelines"
    }
    
    actor "Data Analyst" {
        id: "ANA-001"
        description: "Analyzes data"
    }
    
    actor "DBA" {
        id: "DBA-001"
        description: "Database admin"
    }
    
    operational_activity "Extract Data" {
        id: "OA-001"
        performed_by: "ENG-001"
        description: "Extract from Oracle"
    }
    
    operational_activity "Transform Data" {
        id: "OA-002"
        performed_by: "ENG-001"
        description: "Transform to Delta"
    }
    
    operational_activity "Validate Migration" {
        id: "OA-003"
        performed_by: "DBA-001"
        description: "Validate integrity"
    }
    
    operational_capability "Zero Downtime Migration" {
        id: "CAP-001"
        description: "Migrate without interruption"
    }
}

system_analysis "Migration Platform" {
    requirement "REQ-001" {
        description: "Migrate all Oracle tables to Delta"
        priority: Critical
    }
    
    requirement "REQ-002" {
        description: "Process 1TB per hour"
        priority: High
    }
    
    requirement "REQ-003" {
        description: "Encrypt data in transit"
        priority: Critical
    }
    
    system_function "Extract" {
        id: "SF-001"
        description: "Extract from Oracle"
    }
    
    system_function "Transform" {
        id: "SF-002"
        description: "Transform to Delta"
    }
    
    system_function "Load" {
        id: "SF-003"
        description: "Load to Databricks"
    }
    
    system_function "Validate" {
        id: "SF-004"
        description: "Validate quality"
    }
}

logical_architecture "Migration Architecture" {
    component "OracleConnector" {
        name: "Oracle Connector"
        type: "Service"
        description: "Connects to Oracle databases"
        
        function "Connect" {
            id: "LF-001"
        }
        
        function "Extract" {
            id: "LF-002"
        }
        
        function "Stream" {
            id: "LF-003"
        }
    }
    
    component "ETLEngine" {
        name: "ETL Engine"
        type: "Process"
        description: "Transforms data to Delta"
        
        function "Parse" {
            id: "LF-004"
        }
        
        function "Transform" {
            id: "LF-005"
        }
        
        function "Optimize" {
            id: "LF-006"
        }
    }
    
    component "DeltaStorage" {
        name: "Delta Lake"
        type: "DataStore"
        description: "Delta Lake storage"
        
        function "Store" {
            id: "LF-007"
        }
        
        function "Version" {
            id: "LF-008"
        }
    }
    
    component "Validator" {
        name: "Data Validator"
        type: "Service"
        description: "Validates data quality"
        
        function "Compare" {
            id: "LF-009"
        }
        
        function "Validate" {
            id: "LF-010"
        }
    }
    
    component "Orchestrator" {
        name: "Migration Orchestrator"
        type: "Service"
        description: "Orchestrates workflow"
        
        function "Schedule" {
            id: "LF-011"
        }
        
        function "Monitor" {
            id: "LF-012"
        }
    }
    
    connection "OracleConnector" -> "ETLEngine" {
        label: "raw_data"
    }
    
    connection "ETLEngine" -> "DeltaStorage" {
        label: "delta_tables"
    }
    
    connection "DeltaStorage" -> "Validator" {
        label: "validation_data"
    }
    
    connection "Orchestrator" -> "OracleConnector" {
        label: "control"
    }
    
    connection "Orchestrator" -> "ETLEngine" {
        label: "control"
    }
}

physical_architecture "AWS Deployment" {
    node "OracleDB" {
        id: "HW-001"
        type: hardware
        description: "Oracle database"
    }
    
    node "EC2Connector" {
        id: "HW-002"
        type: hardware
        description: "EC2 connector instance"
    }
    
    node "DatabricksCluster" {
        id: "HW-003"
        type: hardware
        description: "Databricks cluster"
    }
    
    node "S3Storage" {
        id: "HW-004"
        type: hardware
        description: "S3 Delta Lake"
    }
    
    link "HW-001" -> "HW-002" {
        type: network
        description: "VPN tunnel"
    }
    
    link "HW-002" -> "HW-003" {
        type: network
        description: "AWS network"
    }
    
    link "HW-003" -> "HW-004" {
        type: network
        description: "Storage link"
    }
    
    deployment "OracleConnector" -> "HW-002"
    deployment "ETLEngine" -> "HW-003"
    deployment "DeltaStorage" -> "HW-004"
}
