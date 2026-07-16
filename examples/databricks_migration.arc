// Databricks Migration Architecture - Oracle to Databricks
// MBSE Model following Arcadia 7 Dimensions methodology

// ========================================
// 1. OPERATIONAL ANALYSIS - Business Context
// ========================================

operational_analysis "Databricks Migration" {
    // Business Actors
    actor "Data Analyst" {
        id: "OA-ACT-001"
        description: "Analyzes business data and creates reports"
        responsibilities: ["Run SQL queries", "Create dashboards", "Generate reports"]
    }
    
    actor "Data Engineer" {
        id: "OA-ACT-002"
        description: "Manages data pipelines and ETL processes"
        responsibilities: ["Build ETL pipelines", "Monitor data quality", "Optimize performance"]
    }
    
    actor "Database Administrator" {
        id: "OA-ACT-003"
        description: "Manages database infrastructure"
        responsibilities: ["Database maintenance", "Performance tuning", "Backup/Recovery"]
    }
    
    actor "Business User" {
        id: "OA-ACT-004"
        description: "Consumes data insights and reports"
        responsibilities: ["View reports", "Request analytics", "Make business decisions"]
    }
    
    // Operational Activities
    operational_activity "Query Historical Data" {
        id: "OA-ACT-101"
        performed_by: "OA-ACT-001"
        description: "Access and analyze historical business data"
    }
    
    operational_activity "Transform Data" {
        id: "OA-ACT-102"
        performed_by: "OA-ACT-002"
        description: "Clean, transform and prepare data for analysis"
    }
    
    operational_activity "Monitor System Health" {
        id: "OA-ACT-103"
        performed_by: "OA-ACT-003"
        description: "Monitor database performance and availability"
    }
    
    operational_activity "Generate Business Reports" {
        id: "OA-ACT-104"
        performed_by: "OA-ACT-001"
        description: "Create dashboards and analytics reports"
    }
    
    // Operational Capabilities
    operational_capability "Data Migration" {
        id: "OA-CAP-001"
        description: "Migrate data from Oracle to Databricks with zero downtime"
        activities: ["OA-ACT-101", "OA-ACT-102"]
    }
    
    operational_capability "Analytics & Reporting" {
        id: "OA-CAP-002"
        description: "Provide real-time analytics and reporting capabilities"
        activities: ["OA-ACT-104"]
    }
}

// ========================================
// 2. SYSTEM ANALYSIS - System Requirements
// ========================================

system_analysis "Migration System" {
    // System Requirements
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
        description: "System shall maintain 99.9% uptime during migration"
        priority: High
        verification_method: "Test"
    }
    
    requirement "REQ-005" {
        description: "System shall encrypt all data in transit and at rest"
        priority: Critical
        verification_method: "Inspection"
    }
    
    // System Functions
    system_function "Extract Data" {
        id: "SF-001"
        description: "Extract data from Oracle source databases"
        inputs: ["Oracle Connection String", "Table Definitions"]
        outputs: ["Raw Data Streams"]
    }
    
    system_function "Transform Data" {
        id: "SF-002"
        description: "Transform data to Delta Lake format"
        inputs: ["Raw Data Streams"]
        outputs: ["Delta Lake Tables"]
    }
    
    system_function "Load Data" {
        id: "SF-003"
        description: "Load transformed data into Databricks"
        inputs: ["Delta Lake Tables"]
        outputs: ["Loaded Tables Status"]
    }
    
    system_function "Validate Data" {
        id: "SF-004"
        description: "Validate data integrity and quality"
        inputs: ["Loaded Tables Status"]
        outputs: ["Validation Report"]
    }
    
    system_function "Monitor Migration" {
        id: "SF-005"
        description: "Monitor migration progress and health"
        inputs: ["System Metrics"]
        outputs: ["Health Dashboard"]
    }
}

// ========================================
// 3. LOGICAL ARCHITECTURE - Solution Design
// ========================================

logical_architecture "Migration Architecture" {
    // Logical Components
    component "Oracle Connector" {
        id: "LC-001"
        type: "Service"
        description: "Connects to Oracle databases and extracts data"
        
        interface_in "Configuration" {
            protocol: "JDBC"
            format: "Connection String"
        }
        
        interface_out "Data Stream" {
            protocol: "Kafka"
            format: "Avro"
            bandwidth: "100 MB/s"
        }
        
        function "Connect to Oracle"
        function "Query Tables"
        function "Stream Data"
    }
    
    component "ETL Engine" {
        id: "LC-002"
        type: "Process"
        description: "Transforms data from Oracle to Delta format"
        
        interface_in "Raw Data" {
            protocol: "Kafka"
            format: "Avro"
        }
        
        interface_out "Delta Tables" {
            protocol: "S3"
            format: "Parquet"
        }
        
        function "Parse Schema"
        function "Transform Records"
        function "Optimize Partitions"
        function "Write Delta"
    }
    
    component "Delta Lake Storage" {
        id: "LC-003"
        type: "DataStore"
        description: "Stores data in Delta Lake format on S3"
        
        interface_in "Write Data" {
            protocol: "S3"
            format: "Parquet"
        }
        
        interface_out "Read Data" {
            protocol: "S3"
            format: "Parquet"
        }
        
        function "Store Tables"
        function "Version Control"
        function "ACID Transactions"
    }
    
    component "Data Validator" {
        id: "LC-004"
        type: "Service"
        description: "Validates data quality and integrity"
        
        interface_in "Source Data" {
            protocol: "JDBC"
            format: "SQL"
        }
        
        interface_in "Target Data" {
            protocol: "S3"
            format: "Parquet"
        }
        
        interface_out "Validation Results" {
            protocol: "REST"
            format: "JSON"
        }
        
        function "Compare Schemas"
        function "Validate Counts"
        function "Check Data Quality"
    }
    
    component "Migration Orchestrator" {
        id: "LC-005"
        type: "Service"
        description: "Orchestrates migration workflow"
        
        interface_in "Migration Config" {
            protocol: "REST"
            format: "JSON"
        }
        
        interface_out "Control Commands" {
            protocol: "Kafka"
            format: "JSON"
        }
        
        function "Schedule Jobs"
        function "Monitor Progress"
        function "Handle Failures"
    }
    
    component "Monitoring Dashboard" {
        id: "LC-006"
        type: "UI"
        description: "Visualizes migration progress and metrics"
        
        interface_in "Metrics Stream" {
            protocol: "WebSocket"
            format: "JSON"
        }
        
        function "Display Progress"
        function "Show Alerts"
        function "Generate Reports"
    }
    
    // Logical Connections
    connection "LC-001" -> "LC-002" {
        label: "Raw data stream"
    }
    
    connection "LC-002" -> "LC-003" {
        label: "Delta tables"
    }
    
    connection "LC-003" -> "LC-004" {
        label: "Target data"
    }
    
    connection "LC-005" -> "LC-001" {
        label: "Start extraction"
    }
    
    connection "LC-005" -> "LC-002" {
        label: "Start transformation"
    }
    
    connection "LC-004" -> "LC-006" {
        label: "Validation results"
    }
}

// ========================================
// 4. PHYSICAL ARCHITECTURE - Deployment
// ========================================

physical_architecture "Cloud Deployment" {
    // Physical Nodes
    node "Oracle Database Server" {
        id: "PN-001"
        type: "hardware"
        description: "On-premise Oracle database"
        properties: {
            location: "Data Center"
            cpu: "32 cores"
            memory: "256 GB"
            storage: "10 TB SSD"
        }
    }
    
    node "AWS EC2 Connector" {
        id: "PN-002"
        type: "hardware"
        description: "EC2 instance running Oracle connector"
        properties: {
            instance_type: "m5.4xlarge"
            cpu: "16 vCPU"
            memory: "64 GB"
        }
    }
    
    node "Databricks Cluster" {
        id: "PN-003"
        type: "hardware"
        description: "Databricks compute cluster"
        properties: {
            worker_type: "i3.2xlarge"
            workers: "10"
            driver_type: "i3.4xlarge"
        }
    }
    
    node "S3 Bucket" {
        id: "PN-004"
        type: "hardware"
        description: "S3 storage for Delta Lake"
        properties: {
            region: "us-east-1"
            storage_class: "Standard"
        }
    }
    
    node "Kafka Cluster" {
        id: "PN-005"
        type: "hardware"
        description: "MSK Kafka cluster for data streaming"
        properties: {
            brokers: "3"
            instance_type: "kafka.m5.large"
        }
    }
    
    // Physical Links
    link "PN-001" -> "PN-002" {
        type: "VPN"
        description: "Secure VPN connection"
        bandwidth: "10 Gbps"
    }
    
    link "PN-002" -> "PN-005" {
        type: "Network"
        description: "AWS internal network"
        bandwidth: "25 Gbps"
    }
    
    link "PN-005" -> "PN-003" {
        type: "Network"
        description: "Kafka to Databricks"
        bandwidth: "25 Gbps"
    }
    
    link "PN-003" -> "PN-004" {
        type: "Storage"
        description: "Databricks to S3"
        bandwidth: "100 Gbps"
    }
    
    // Component Deployment
    deployment "LC-001" -> "PN-002" {
        description: "Oracle Connector on EC2"
    }
    
    deployment "LC-002" -> "PN-003" {
        description: "ETL Engine on Databricks"
    }
    
    deployment "LC-003" -> "PN-004" {
        description: "Delta Lake on S3"
    }
    
    deployment "LC-005" -> "PN-003" {
        description: "Orchestrator on Databricks"
    }
}

// ========================================
// 5. EPBS - Product Breakdown
// ========================================

epbs "Migration Product Structure" {
    subsystem "Source System" {
        id: "EPBS-001"
        description: "Oracle database and connectors"
    }
    
    subsystem "Migration Platform" {
        id: "EPBS-002"
        description: "ETL and orchestration components"
        
        assembly "Data Ingestion Layer" {
            id: "EPBS-002-01"
            components: ["Oracle Connector", "Kafka Streams"]
        }
        
        assembly "Transformation Layer" {
            id: "EPBS-002-02"
            components: ["ETL Engine", "Data Validators"]
        }
        
        assembly "Orchestration Layer" {
            id: "EPBS-002-03"
            components: ["Workflow Engine", "Monitoring Service"]
        }
    }
    
    subsystem "Target System" {
        id: "EPBS-003"
        description: "Databricks platform and storage"
        
        assembly "Compute Layer" {
            id: "EPBS-003-01"
            components: ["Databricks Clusters", "Spark Jobs"]
        }
        
        assembly "Storage Layer" {
            id: "EPBS-003-02"
            components: ["Delta Lake", "S3 Buckets"]
        }
    }
}

// ========================================
// 6. REQUIREMENTS TRACEABILITY
// ========================================

// Trace requirements to components
trace "LC-001" satisfies "REQ-001" {
    rationale: "Oracle Connector extracts all tables"
}

trace "LC-002" satisfies "REQ-001" {
    rationale: "ETL Engine transforms to Delta format"
}

trace "LC-005" satisfies "REQ-002" {
    rationale: "Orchestrator enables zero-downtime migration"
}

trace "LC-002" satisfies "REQ-003" {
    rationale: "ETL Engine optimized for 1TB/hour throughput"
}

trace "LC-005" satisfies "REQ-004" {
    rationale: "Orchestrator monitors and ensures 99.9% uptime"
}

// Trace functions to requirements
trace "SF-001" satisfies "REQ-001" {
    rationale: "Extract function retrieves all Oracle data"
}

trace "SF-002" satisfies "REQ-001" {
    rationale: "Transform function converts to Delta"
}

trace "SF-004" satisfies "REQ-001" {
    rationale: "Validate function ensures data integrity"
}

// ========================================
// 7. CROSS-CUTTING CONCERNS
// ========================================

// Security Policies
security_policy "Encryption in Transit" {
    id: "SEC-001"
    description: "All data transfers must use TLS 1.3"
    applies_to: ["LC-001", "LC-002", "LC-003"]
}

security_policy "Encryption at Rest" {
    id: "SEC-002"
    description: "All stored data must be encrypted with AES-256"
    applies_to: ["LC-003", "PN-004"]
}

security_policy "Access Control" {
    id: "SEC-003"
    description: "All services must use IAM role-based access"
    applies_to: ["LC-001", "LC-002", "LC-004", "LC-005"]
}

// Safety Constraints
safety_constraint "Data Backup" {
    id: "SAFE-001"
    description: "Maintain continuous backup during migration"
    criticality: "High"
}

safety_constraint "Rollback Capability" {
    id: "SAFE-002"
    description: "Support instant rollback in case of failure"
    criticality: "Critical"
}

// Performance Metrics
performance_metric "Migration Speed" {
    id: "PERF-001"
    target: "1 TB/hour"
    measurement: "Throughput monitoring"
}

performance_metric "Query Latency" {
    id: "PERF-002"
    target: "< 100ms p99"
    measurement: "Query performance tracking"
}

performance_metric "System Availability" {
    id: "PERF-003"
    target: "99.9%"
    measurement: "Uptime monitoring"
}
