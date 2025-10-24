// Pluxee Analytics Business Requirements Model
// This model captures the analytics requirements and their relationships

operational_analysis "Pluxee Analytics Operations" {
    actor "Executive" {
        id: "ACT-001"
        description: "Executive leadership requiring consolidated analytics"
        concerns: ["KPI monitoring", "Program performance", "Strategic decisions"]
    }
    
    actor "Market Analyst" {
        id: "ACT-002"
        description: "Analyzes market trends and consumer behavior"
        concerns: ["Transaction patterns", "Consumer behavior", "Market insights"]
    }
    
    actor "Merchant Manager" {
        id: "ACT-003"
        description: "Manages merchant relationships and network"
        concerns: ["Merchant performance", "Network effectiveness"]
    }
    
    actor "Client Manager" {
        id: "ACT-004"
        description: "Manages client portfolios and programs"
        concerns: ["Client programs", "Portfolio performance"]
    }
    
    actor "Finance Team" {
        id: "ACT-005"
        description: "Financial analysis and reporting"
        concerns: ["Revenue analysis", "Cost patterns", "Financial KPIs"]
    }
    
    actor "Compliance Officer" {
        id: "ACT-006"
        description: "Ensures regulatory compliance"
        concerns: ["Regulatory reporting", "Compliance metrics"]
    }
    
    actor "Data Quality Manager" {
        id: "ACT-007"
        description: "Ensures data quality and lineage"
        concerns: ["Data quality", "Data lineage", "Data governance"]
    }
}

system_analysis "Analytics Platform Requirements" {
    // Executive Analytics Requirements
    requirement "BR-ANALYTICS-001" {
        id: "BR-ANALYTICS-001"
        name: "Consolidated KPI Monitoring Across Markets"
        description: "System shall provide consolidated KPI monitoring across all markets with real-time dashboards"
        priority: "Critical"
        category: "Executive Analytics"
    }
    
    requirement "BR-ANALYTICS-002" {
        id: "BR-ANALYTICS-002"
        name: "Benefit Program Performance Analysis"
        description: "System shall analyze benefit program performance across client base"
        priority: "Critical"
        category: "Executive Analytics"
    }
    
    // Market Analytics Requirements
    requirement "BR-ANALYTICS-003" {
        id: "BR-ANALYTICS-003"
        name: "Transaction Pattern Analysis"
        description: "System shall analyze transaction patterns to identify trends and anomalies"
        priority: "High"
        category: "Market Analytics"
    }
    
    requirement "BR-ANALYTICS-004" {
        id: "BR-ANALYTICS-004"
        name: "Consumer Behavior Tracking"
        description: "System shall track and analyze consumer behavior patterns"
        priority: "High"
        category: "Market Analytics"
    }
    
    // Merchant Analytics Requirements
    requirement "BR-ANALYTICS-005" {
        id: "BR-ANALYTICS-005"
        name: "Merchant Performance Analysis"
        description: "System shall analyze individual merchant performance metrics"
        priority: "High"
        category: "Merchant Analytics"
    }
    
    requirement "BR-ANALYTICS-006" {
        id: "BR-ANALYTICS-006"
        name: "Merchant Network Effectiveness Tracking"
        description: "System shall track overall merchant network effectiveness"
        priority: "High"
        category: "Merchant Analytics"
    }
    
    // Client Analytics Requirements
    requirement "BR-ANALYTICS-007" {
        id: "BR-ANALYTICS-007"
        name: "Client Program Analysis"
        description: "System shall analyze client program performance and engagement"
        priority: "High"
        category: "Client Analytics"
    }
    
    requirement "BR-ANALYTICS-008" {
        id: "BR-ANALYTICS-008"
        name: "Client Portfolio Performance Tracking"
        description: "System shall track portfolio performance across all clients"
        priority: "High"
        category: "Client Analytics"
    }
    
    // Financial Analytics Requirements
    requirement "BR-ANALYTICS-009" {
        id: "BR-ANALYTICS-009"
        name: "Revenue and Cost Pattern Analysis"
        description: "System shall analyze revenue and cost patterns for financial insights"
        priority: "Critical"
        category: "Financial Analytics"
    }
    
    requirement "BR-ANALYTICS-010" {
        id: "BR-ANALYTICS-010"
        name: "Financial KPI Tracking Across Markets"
        description: "System shall track financial KPIs across all markets"
        priority: "Critical"
        category: "Financial Analytics"
    }
    
    // Compliance Analytics Requirements
    requirement "BR-ANALYTICS-011" {
        id: "BR-ANALYTICS-011"
        name: "Regulatory Reporting"
        description: "System shall generate regulatory reports for compliance"
        priority: "Critical"
        category: "Compliance Analytics"
    }
    
    requirement "BR-ANALYTICS-012" {
        id: "BR-ANALYTICS-012"
        name: "Compliance Metric Tracking"
        description: "System shall track compliance metrics continuously"
        priority: "Critical"
        category: "Compliance Analytics"
    }
    
    // Data Quality Analytics Requirements
    requirement "BR-ANALYTICS-013" {
        id: "BR-ANALYTICS-013"
        name: "Data Quality Monitoring"
        description: "System shall monitor data quality metrics across all data sources"
        priority: "High"
        category: "Data Quality Analytics"
    }
    
    requirement "BR-ANALYTICS-014" {
        id: "BR-ANALYTICS-014"
        name: "Data Lineage Tracking"
        description: "System shall track data lineage from source to analytics"
        priority: "High"
        category: "Data Quality Analytics"
    }
    
    // Advanced Analytics Requirements
    requirement "BR-ADV-001" {
        id: "BR-ADV-001"
        name: "Predictive Analytics Capabilities"
        description: "System shall provide predictive analytics using ML models"
        priority: "Medium"
        category: "Advanced Analytics"
    }
    
    requirement "BR-ADV-002" {
        id: "BR-ADV-002"
        name: "Prescriptive Analytics Capabilities"
        description: "System shall provide prescriptive recommendations based on analytics"
        priority: "Medium"
        category: "Advanced Analytics"
    }
    
    // Implementation Requirements
    requirement "BR-ARCH-001" {
        id: "BR-ARCH-001"
        name: "Efficient Analytics Processing"
        description: "System shall process analytics queries efficiently at scale"
        priority: "High"
        category: "Implementation"
    }
    
    requirement "BR-GOV-001" {
        id: "BR-GOV-001"
        name: "Data Quality for Analytics"
        description: "System shall ensure data quality standards for analytics"
        priority: "Critical"
        category: "Implementation"
    }
}

logical_architecture "Analytics Platform Architecture" {
    component "Executive Dashboard" {
        id: "LC-001"
        type: "Logical"
        category: "Executive Analytics"
        
        function "Aggregate KPIs" {
            id: "LF-001"
            description: "Aggregates KPIs from all markets"
        }
        
        function "Analyze Program Performance" {
            id: "LF-002"
            description: "Analyzes benefit program performance"
        }
    }
    
    component "Market Analytics Engine" {
        id: "LC-002"
        type: "Logical"
        category: "Market Analytics"
        
        function "Analyze Transaction Patterns" {
            id: "LF-003"
            description: "Analyzes transaction patterns"
        }
        
        function "Track Consumer Behavior" {
            id: "LF-004"
            description: "Tracks consumer behavior"
        }
    }
    
    component "Merchant Analytics Engine" {
        id: "LC-003"
        type: "Logical"
        category: "Merchant Analytics"
        
        function "Analyze Merchant Performance" {
            id: "LF-005"
            description: "Analyzes merchant performance"
        }
        
        function "Track Network Effectiveness" {
            id: "LF-006"
            description: "Tracks merchant network effectiveness"
        }
    }
    
    component "Client Analytics Engine" {
        id: "LC-004"
        type: "Logical"
        category: "Client Analytics"
        
        function "Analyze Client Programs" {
            id: "LF-007"
            description: "Analyzes client programs"
        }
        
        function "Track Portfolio Performance" {
            id: "LF-008"
            description: "Tracks client portfolio performance"
        }
    }
    
    component "Financial Analytics Engine" {
        id: "LC-005"
        type: "Logical"
        category: "Financial Analytics"
        
        function "Analyze Revenue Patterns" {
            id: "LF-009"
            description: "Analyzes revenue and cost patterns"
        }
        
        function "Track Financial KPIs" {
            id: "LF-010"
            description: "Tracks financial KPIs"
        }
    }
    
    component "Compliance Analytics Engine" {
        id: "LC-006"
        type: "Logical"
        category: "Compliance Analytics"
        
        function "Generate Regulatory Reports" {
            id: "LF-011"
            description: "Generates regulatory reports"
        }
        
        function "Track Compliance Metrics" {
            id: "LF-012"
            description: "Tracks compliance metrics"
        }
    }
    
    component "Data Quality Engine" {
        id: "LC-007"
        type: "Logical"
        category: "Data Quality Analytics"
        
        function "Monitor Data Quality" {
            id: "LF-013"
            description: "Monitors data quality"
        }
        
        function "Track Data Lineage" {
            id: "LF-014"
            description: "Tracks data lineage"
        }
    }
    
    component "Advanced Analytics Engine" {
        id: "LC-008"
        type: "Logical"
        category: "Advanced Analytics"
        
        function "Predictive Analytics" {
            id: "LF-015"
            description: "Performs predictive analytics"
        }
        
        function "Prescriptive Analytics" {
            id: "LF-016"
            description: "Provides prescriptive recommendations"
        }
    }
    
    component "Analytics Infrastructure" {
        id: "LC-009"
        type: "Logical"
        category: "Implementation"
        
        function "Process Analytics Queries" {
            id: "LF-017"
            description: "Processes analytics queries efficiently"
        }
        
        function "Ensure Data Quality" {
            id: "LF-018"
            description: "Ensures data quality for analytics"
        }
    }
}

// Traceability: Requirements to Components
trace "LC-001" satisfies "BR-ANALYTICS-001" {
    rationale: "Executive Dashboard provides consolidated KPI monitoring"
}

trace "LC-001" satisfies "BR-ANALYTICS-002" {
    rationale: "Executive Dashboard analyzes program performance"
}

trace "LC-002" satisfies "BR-ANALYTICS-003" {
    rationale: "Market Analytics Engine analyzes transaction patterns"
}

trace "LC-002" satisfies "BR-ANALYTICS-004" {
    rationale: "Market Analytics Engine tracks consumer behavior"
}

trace "LC-003" satisfies "BR-ANALYTICS-005" {
    rationale: "Merchant Analytics Engine analyzes merchant performance"
}

trace "LC-003" satisfies "BR-ANALYTICS-006" {
    rationale: "Merchant Analytics Engine tracks network effectiveness"
}

trace "LC-004" satisfies "BR-ANALYTICS-007" {
    rationale: "Client Analytics Engine analyzes client programs"
}

trace "LC-004" satisfies "BR-ANALYTICS-008" {
    rationale: "Client Analytics Engine tracks portfolio performance"
}

trace "LC-005" satisfies "BR-ANALYTICS-009" {
    rationale: "Financial Analytics Engine analyzes revenue patterns"
}

trace "LC-005" satisfies "BR-ANALYTICS-010" {
    rationale: "Financial Analytics Engine tracks financial KPIs"
}

trace "LC-006" satisfies "BR-ANALYTICS-011" {
    rationale: "Compliance Engine generates regulatory reports"
}

trace "LC-006" satisfies "BR-ANALYTICS-012" {
    rationale: "Compliance Engine tracks compliance metrics"
}

trace "LC-007" satisfies "BR-ANALYTICS-013" {
    rationale: "Data Quality Engine monitors data quality"
}

trace "LC-007" satisfies "BR-ANALYTICS-014" {
    rationale: "Data Quality Engine tracks data lineage"
}

trace "LC-008" satisfies "BR-ADV-001" {
    rationale: "Advanced Analytics Engine provides predictive capabilities"
}

trace "LC-008" satisfies "BR-ADV-002" {
    rationale: "Advanced Analytics Engine provides prescriptive recommendations"
}

trace "LC-009" satisfies "BR-ARCH-001" {
    rationale: "Analytics Infrastructure ensures efficient processing"
}

trace "LC-009" satisfies "BR-GOV-001" {
    rationale: "Analytics Infrastructure ensures data quality"
}

// Requirement Dependencies (from your Mermaid diagram)
trace "BR-ANALYTICS-001" implements "BR-ANALYTICS-010" {
    rationale: "Consolidated KPIs depend on financial KPI tracking"
}

trace "BR-ANALYTICS-001" implements "BR-ANALYTICS-003" {
    rationale: "Consolidated KPIs depend on transaction pattern analysis"
}

trace "BR-ANALYTICS-002" implements "BR-ANALYTICS-007" {
    rationale: "Program performance depends on client program analysis"
}

trace "BR-ANALYTICS-002" implements "BR-ANALYTICS-008" {
    rationale: "Program performance depends on portfolio tracking"
}

trace "BR-ANALYTICS-003" implements "BR-ANALYTICS-005" {
    rationale: "Transaction patterns inform merchant performance"
}

trace "BR-ANALYTICS-003" implements "BR-ANALYTICS-006" {
    rationale: "Transaction patterns inform network effectiveness"
}

trace "BR-ANALYTICS-004" implements "BR-ANALYTICS-007" {
    rationale: "Consumer behavior informs client programs"
}

trace "BR-ANALYTICS-004" implements "BR-ADV-001" {
    rationale: "Consumer behavior enables predictive analytics"
}

trace "BR-ANALYTICS-005" implements "BR-ANALYTICS-006" {
    rationale: "Merchant performance feeds network effectiveness"
}

trace "BR-ANALYTICS-005" implements "BR-ANALYTICS-009" {
    rationale: "Merchant performance impacts revenue analysis"
}

trace "BR-ANALYTICS-006" implements "BR-ADV-002" {
    rationale: "Network effectiveness enables prescriptive analytics"
}

trace "BR-ANALYTICS-007" implements "BR-ANALYTICS-008" {
    rationale: "Client programs feed portfolio tracking"
}

trace "BR-ANALYTICS-007" implements "BR-ANALYTICS-002" {
    rationale: "Client programs feed benefit program analysis"
}

trace "BR-ANALYTICS-008" implements "BR-ANALYTICS-009" {
    rationale: "Portfolio performance impacts revenue analysis"
}

trace "BR-ANALYTICS-009" implements "BR-ANALYTICS-010" {
    rationale: "Revenue patterns feed financial KPIs"
}

trace "BR-ANALYTICS-009" implements "BR-ADV-001" {
    rationale: "Revenue patterns enable predictive analytics"
}

trace "BR-ANALYTICS-010" implements "BR-ADV-002" {
    rationale: "Financial KPIs enable prescriptive recommendations"
}

trace "BR-ANALYTICS-011" implements "BR-ANALYTICS-012" {
    rationale: "Regulatory reporting depends on compliance metrics"
}

trace "BR-ANALYTICS-011" implements "BR-ANALYTICS-013" {
    rationale: "Regulatory reporting depends on data quality"
}

trace "BR-ANALYTICS-012" implements "BR-GOV-001" {
    rationale: "Compliance metrics require data quality governance"
}

trace "BR-ANALYTICS-013" implements "BR-GOV-001" {
    rationale: "Data quality monitoring enforces governance"
}

trace "BR-ANALYTICS-013" implements "BR-ANALYTICS-014" {
    rationale: "Data quality depends on lineage tracking"
}

trace "BR-ANALYTICS-014" implements "BR-ARCH-001" {
    rationale: "Data lineage supports efficient processing"
}

trace "BR-ADV-001" implements "BR-ADV-002" {
    rationale: "Predictive analytics enables prescriptive recommendations"
}

trace "BR-ARCH-001" implements "BR-ADV-001" {
    rationale: "Efficient processing enables predictive analytics"
}

trace "BR-ARCH-001" implements "BR-ADV-002" {
    rationale: "Efficient processing enables prescriptive analytics"
}

trace "BR-GOV-001" implements "BR-ANALYTICS-013" {
    rationale: "Data governance ensures quality monitoring"
}
