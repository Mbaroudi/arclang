operational_analysis "Tactical Mission System" {
    actor "Pilot" {
        id: "ACT-001"
        description: "Fighter aircraft pilot"
        classification: "SECRET"
        interactions: ["plan_mission", "execute_maneuvers", "engage_targets"]
    }
    
    actor "Weapons System Officer" {
        id: "ACT-002"
        description: "Back-seater managing sensors and weapons"
        classification: "SECRET"
        interactions: ["operate_radar", "designate_targets", "control_weapons"]
    }
    
    actor "Ground Control" {
        id: "ACT-003"
        description: "Command and control center"
        classification: "TOP_SECRET"
        interactions: ["provide_intelligence", "coordinate_missions", "authorize_engagement"]
    }
    
    operational_capability "Air-to-Air Combat" {
        id: "OC-001"
        description: "Detect, track, and engage airborne threats"
        classification: "SECRET"
        involving: ["Pilot", "Weapons System Officer"]
        phases: ["detection", "tracking", "engagement", "assessment"]
    }
    
    operational_capability "Air-to-Ground Strike" {
        id: "OC-002"
        description: "Identify and engage ground targets with precision"
        classification: "SECRET"
        involving: ["Pilot", "Weapons System Officer", "Ground Control"]
        authorization_required: true
    }
    
    operational_activity "Track Multiple Targets" {
        id: "OA-001"
        description: "Maintain situational awareness of multiple threats"
        performed_by: "Weapons System Officer"
        max_targets: 20
        update_rate: "1Hz"
    }
    
    operational_activity "Weapon Employment" {
        id: "OA-002"
        description: "Select and launch appropriate ordnance"
        performed_by: "Pilot"
        safety_critical: true
        requires_authorization: true
    }
}

system_analysis "Mission Computer System" {
    requirement "SYS-MC-001" {
        description: "The mission computer shall process radar data with latency less than 100ms"
        priority: "Critical"
        safety_level: "DAL_A"
        classification: "SECRET"
        traces: ["OA-001"]
        verification_method: "Test"
    }
    
    requirement "SYS-MC-002" {
        description: "The system shall maintain track continuity for all detected targets"
        priority: "Critical"
        safety_level: "DAL_A"
        performance_metric: "99.9 percent track continuity"
    }
    
    requirement "SYS-MC-003" {
        description: "Weapon release authorization shall require dual operator confirmation"
        priority: "Critical"
        safety_level: "DAL_A"
        security_level: "Type 1 Encryption"
        rationale: "Prevent unauthorized weapon release"
    }
    
    requirement "SYS-MC-004" {
        description: "All tactical data shall be encrypted using NSA-approved algorithms"
        priority: "Critical"
        classification: "TOP_SECRET"
        standard: "FIPS 140-3"
    }
    
    requirement "SYS-MC-005" {
        description: "System shall operate in contested electromagnetic environment"
        priority: "High"
        emcon_level: 3
        jamming_resistance: "High"
    }
    
    requirement "SYS-MC-006" {
        description: "Mission data shall be stored with tamper detection"
        priority: "High"
        security_level: "Integrity Protected"
        audit_trail: true
    }
    
    system_function "Sensor Management" {
        id: "SF-001"
        description: "Control and coordinate onboard sensors"
        inputs: ["operator_commands", "threat_priorities"]
        outputs: ["sensor_tasks", "raw_sensor_data"]
        safety_level: "DAL_A"
        classification: "SECRET"
    }
    
    system_function "Track Management" {
        id: "SF-002"
        description: "Correlate and maintain tracks on detected objects"
        inputs: ["raw_sensor_data", "datalink_tracks"]
        outputs: ["fused_track_file"]
        safety_level: "DAL_A"
        max_tracks: 200
    }
    
    system_function "Threat Evaluation" {
        id: "SF-003"
        description: "Assess threat level and prioritize responses"
        inputs: ["fused_track_file", "threat_library"]
        outputs: ["threat_priorities", "recommended_actions"]
        classification: "TOP_SECRET"
    }
    
    system_function "Weapon Control" {
        id: "SF-004"
        description: "Manage weapon inventory and employment"
        inputs: ["target_designation", "authorization_code"]
        outputs: ["weapon_release_command"]
        safety_level: "DAL_A"
        security_critical: true
    }
}

logical_architecture "Mission Computer Logical Architecture" {
    component "Radar Processor" {
        id: "LC-001"
        type: "Logical"
        description: "AESA radar signal processor"
        classification: "SECRET"
        
        function "Beam Steering" {
            id: "LF-001"
            inputs: ["steering_commands"]
            outputs: ["rf_pattern"]
            scan_rate: "10 scans per second"
        }
        
        function "Target Detection" {
            id: "LF-002"
            inputs: ["return_signals"]
            outputs: ["raw_detections"]
            algorithm: "CFAR with Doppler processing"
            classification: "TOP_SECRET"
        }
        
        function "Doppler Processing" {
            id: "LF-003"
            inputs: ["return_signals"]
            outputs: ["velocity_estimates"]
            prf: "Variable 10-100 kHz"
        }
    }
    
    component "Electronic Warfare Processor" {
        id: "LC-002"
        type: "Logical"
        description: "Threat warning and countermeasures"
        classification: "TOP_SECRET"
        
        function "RWR Processing" {
            id: "LF-004"
            description: "Radar warning receiver processing"
            inputs: ["rf_intercepts"]
            outputs: ["threat_warnings"]
            frequency_range: "2-18 GHz"
        }
        
        function "Countermeasures Control" {
            id: "LF-005"
            inputs: ["threat_warnings"]
            outputs: ["cm_commands"]
            response_time: "less than 1 second"
        }
    }
    
    component "Track Fusion" {
        id: "LC-003"
        type: "Logical"
        description: "Multi-sensor track correlation and fusion"
        safety_level: "DAL_A"
        classification: "SECRET"
        
        function "Track Association" {
            id: "LF-006"
            inputs: [
                "radar_tracks",
                "ew_tracks",
                "datalink_tracks"
            ]
            outputs: ["correlated_tracks"]
            algorithm: "Multi-hypothesis tracking"
        }
        
        function "Track Prediction" {
            id: "LF-007"
            inputs: ["correlated_tracks"]
            outputs: ["predicted_positions"]
            algorithm: "Extended Kalman Filter"
        }
        
        function "Track Quality Assessment" {
            id: "LF-008"
            inputs: ["correlated_tracks"]
            outputs: ["quality_metrics"]
        }
    }
    
    component "Threat Manager" {
        id: "LC-004"
        type: "Logical"
        description: "Threat identification and prioritization"
        classification: "TOP_SECRET"
        
        function "Identify Emitters" {
            id: "LF-009"
            inputs: ["ew_signals", "emitter_library"]
            outputs: ["identified_threats"]
            library_size: "5000 plus emitters"
        }
        
        function "Calculate Threat Priority" {
            id: "LF-010"
            inputs: ["identified_threats", "ownship_state"]
            outputs: ["threat_priorities"]
        }
    }
    
    component "Weapon Manager" {
        id: "LC-005"
        type: "Logical"
        description: "Weapon inventory and employment control"
        safety_level: "DAL_A"
        security_critical: true
        
        function "Weapon Inventory" {
            id: "LF-011"
            outputs: ["available_weapons"]
        }
        
        function "Target-Weapon Pairing" {
            id: "LF-012"
            inputs: ["target", "available_weapons"]
            outputs: ["recommended_weapon"]
            algorithm: "Mission effectiveness optimization"
        }
        
        function "Authorization Check" {
            id: "LF-013"
            inputs: ["release_request", "authorization"]
            outputs: ["authorized"]
            security_level: "Type 1"
        }
        
        function "Generate Release Command" {
            id: "LF-014"
            inputs: ["authorized", "weapon_params"]
            outputs: ["release_command"]
        }
    }
    
    component "Cryptographic Module" {
        id: "LC-006"
        type: "Logical"
        description: "Type 1 encryption for classified data"
        classification: "TOP_SECRET"
        certification: "NSA Type 1"
        
        function "Encrypt Data" {
            id: "LF-015"
            inputs: ["plaintext", "key"]
            outputs: ["ciphertext"]
            algorithm: "AES-256-GCM"
        }
        
        function "Key Management" {
            id: "LF-016"
            description: "Secure key generation and distribution"
            certification: "FIPS 140-3 Level 3"
        }
    }
    
    interface "Sensor Data Bus" {
        id: "LI-001"
        from: "LC-001"
        to: "LC-003"
        iface_type: "Data"
        protocol: "Fibre Channel"
        classification: "SECRET"
        rate: "10Hz"
        bandwidth: "50Mbps"
        latency: "less than 50ms"
    }
    
    interface "Threat Data Bus" {
        id: "LI-002"
        from: "LC-002"
        to: "LC-004"
        iface_type: "Data"
        classification: "TOP_SECRET"
        rate: "20Hz"
        encryption: "Type 1"
    }
    
    interface "Weapon Control Bus" {
        id: "LI-003"
        from: "LC-005"
        to: "Stores Management System"
        iface_type: "Control"
        protocol: "MIL-STD-1760"
        rate: "Event-driven"
        latency: "less than 100ms"
        safety_critical: true
    }
    
}

trace "LC-003" satisfies "SYS-MC-001" {
    rationale: "Track fusion provides required latency performance"
}

trace "LF-013" satisfies "SYS-MC-003" {
    rationale: "Authorization check enforces dual-operator requirement"
}

physical_architecture "Mission Computer Physical Architecture" {
    node "Mission Processor 1" {
        id: "PN-001"
        description: "Primary mission computer"
        processor: "PowerPC e6500 at 2.5 GHz"
        cores: 8
        memory: "32GB ECC RAM"
        safety_level: "DAL_A"
        classification: "TOP_SECRET"
        tempest_certified: true
        
        deploys "LC-003" {
            partition: "Track Fusion Partition"
            criticality: "DAL_A"
            memory_protection: "MMU-enforced"
        }
        
        deploys "LC-004" {
            partition: "Threat Management Partition"
            criticality: "DAL_A"
        }
        
        deploys "LC-005" {
            partition: "Weapon Control Partition"
            criticality: "DAL_A"
            isolation: "Strong spatial and temporal"
        }
    }
    
    node "Mission Processor 2" {
        id: "PN-002"
        description: "Backup mission computer (hot standby)"
        processor: "PowerPC e6500 at 2.5 GHz"
        cores: 8
        memory: "32GB ECC RAM"
        safety_level: "DAL_A"
        
        deploys "LC-003" {
            partition: "Backup Track Fusion"
            mode: "Hot Standby"
        }
    }
    
    node "Radar Signal Processor" {
        id: "PN-003"
        description: "Dedicated radar processing unit"
        processor: "Xilinx Virtex UltraScale+ FPGA"
        classification: "SECRET"
        
        deploys "LC-001"
    }
    
    node "EW Processor" {
        id: "PN-004"
        description: "Electronic warfare processing unit"
        processor: "Multi-core DSP"
        classification: "TOP_SECRET"
        
        deploys "LC-002"
    }
    
    node "Crypto Appliance" {
        id: "PN-005"
        description: "NSA-certified Type 1 encryptor"
        certification: "NSA Type 1, FIPS 140-3 Level 3"
        classification: "TOP_SECRET"
        tamper_protection: "Hardware"
        
        deploys "LC-006"
    }
    
    physical_link "High-Speed Backplane" {
        id: "PL-001"
        topology: "VPX"
        protocol: "Serial RapidIO"
        bandwidth: "10Gbps-per-lane"
        lanes: 4
        classification: "SECRET"
        
        connects: ["PN-001", "PN-002", "PN-003", "PN-004", "PN-005"]
    }
    
    physical_link "Weapon Bus" {
        id: "PL-002"
        protocol: "MIL-STD-1760"
        redundancy: "Dual"
        connects: ["PN-001", "Stores Management"]
    }
    
    physical_link "Cross-Channel Link" {
        id: "PL-003"
        description: "Synchronization between mission processors"
        protocol: "Deterministic Ethernet"
        latency: "less than 1ms"
        connects: ["PN-001", "PN-002"]
    }
}

epbs "Mission Computer EPBS" {
    system "Tactical Mission System" {
        id: "EPBS-001"
        classification: "TOP_SECRET"
        
        subsystem "Mission Processing Subsystem" {
            id: "EPBS-101"
            
            item "Mission Processor Card 1" {
                id: "EPBS-1001"
                part_number: "MPC-001-Rev-F"
                supplier: "Raytheon"
                certification: "DO-254 Level A"
                classification: "TOP_SECRET"
                unit_cost: "$125,000"
            }
            
            item "Mission Processor Card 2" {
                id: "EPBS-1002"
                part_number: "MPC-001-Rev-F"
                supplier: "Raytheon"
                certification: "DO-254 Level A"
                unit_cost: "$125,000"
            }
            
            item "Mission Software" {
                id: "EPBS-1003"
                version: "v7.3.2"
                certification: "DO-178C Level A"
                classification: "TOP_SECRET"
                sloc: 850000
                language: "Ada 2012"
            }
        }
        
        subsystem "Sensor Processing Subsystem" {
            id: "EPBS-102"
            
            item "Radar Signal Processor" {
                id: "EPBS-1004"
                part_number: "RSP-FPGA-001"
                supplier: "Northrop Grumman"
                classification: "SECRET"
                unit_cost: "$85,000"
            }
            
            item "EW Processor" {
                id: "EPBS-1005"
                part_number: "EWP-001"
                supplier: "BAE Systems"
                classification: "TOP_SECRET"
                unit_cost: "$95,000"
            }
        }
        
        subsystem "Security Subsystem" {
            id: "EPBS-103"
            
            item "Type 1 Crypto Module" {
                id: "EPBS-1006"
                part_number: "KIV-77"
                supplier: "General Dynamics"
                certification: "NSA Type 1"
                classification: "TOP_SECRET"
                unit_cost: "$45,000"
                zeroize_capable: true
            }
        }
        
        subsystem "Interconnect Subsystem" {
            id: "EPBS-104"
            
            item "VPX Backplane" {
                id: "EPBS-1007"
                part_number: "VPX-BP-14U"
                specification: "VITA 46"
                slots: 14
            }
            
            item "1760 Interface Card" {
                id: "EPBS-1008"
                part_number: "MS1760-001"
                channels: 8
            }
        }
    }
}

safety_analysis {
    standard: "DO_178C"
    dal: "DAL_A"
    classification: "TOP_SECRET"
    
    hazard "Unintended Weapon Release" {
        id: "HAZ-001"
        description: "Weapon released without proper authorization"
        severity: "Catastrophic"
        likelihood: "ExtremelyImprobable"
        dal: "DAL_A"
        classification: "SECRET"
        
        causes: [
            "Authorization bypass",
            "Software fault in weapon control",
            "Hardware fault in release circuit",
            "Electromagnetic interference"
        ]
        
        mitigations: [
            "Dual-redundant authorization checks",
            "Hardware interlock on weapon release",
            "Software partitioning with MMU enforcement",
            "EMI/EMC hardening per MIL-STD-461"
        ]
    }
    
    hazard "Loss of Track Continuity" {
        id: "HAZ-002"
        description: "Inability to maintain continuous track on threats"
        severity: "Major"
        likelihood: "Remote"
        dal: "DAL_B"
        
        causes: [
            "Sensor processor fault",
            "Track fusion algorithm error",
            "Datalink interruption"
        ]
        
        mitigations: [
            "Redundant mission processors",
            "Multi-sensor fusion",
            "Track coasting algorithm"
        ]
    }
    
    fmea "Mission Computer FMEA" {
        target: "Mission Processor 1"
        failure_mode: "Processor halt"
        effects: "Loss of primary mission functions"
        severity: "Major"
        occurrence: "Remote 1e-6 per hour"
        detection: "Built-in test detects within 100ms"
        rpn: 54
        actions: [
            "Automatic switchover to Processor 2",
            "BIT runs every 10 seconds"
        ]
    }
}
