logical_architecture "Emergency Braking Architecture" {
    component "Main Controller" {
        id: "LC-001"
        type: "Logical"
        level: "LA"
        description: "Central control unit"
        safety_level: "ASIL_D"
    }
    
    component "Sensor Fusion Unit" {
        id: "LC-002"
        type: "Logical"
        level: "LA"
        description: "Multi-sensor fusion processor"
        safety_level: "ASIL_C"
    }
    
    component "Radar Interface" {
        id: "LC-003"
        type: "Logical"
        level: "LA"
        description: "Radar sensor interface"
        safety_level: "ASIL_C"
        
        interface_out "Radar Data" {
            protocol: "CAN"
        }
    }
    
    component "Camera Interface" {
        id: "LC-004"
        type: "Logical"
        level: "LA"
        description: "Camera sensor interface"
        safety_level: "ASIL_C"
        
        interface_out "Image Data" {
            protocol: "Ethernet"
        }
    }
    
    component "Lidar Interface" {
        id: "LC-005"
        type: "Logical"
        level: "LA"
        description: "Lidar sensor interface"
        safety_level: "ASIL_C"
        
        interface_out "Point Cloud" {
            protocol: "Ethernet"
        }
    }
    
    component "Object Tracker" {
        id: "LC-006"
        type: "Logical"
        level: "LA"
        description: "Object tracking unit"
        safety_level: "ASIL_C"
        
        interface_in "Fused Data" {
            protocol: "Internal"
        }
        
        interface_out "Track Data" {
            protocol: "Internal"
        }
    }
    
    component "Risk Assessor" {
        id: "LC-007"
        type: "Logical"
        level: "LA"
        description: "Risk assessment unit"
        safety_level: "ASIL_D"
        
        interface_in "Tracks" {
            protocol: "Internal"
        }
        
        interface_out "Risk Level" {
            protocol: "Internal"
        }
    }
    
    component "Brake Controller" {
        id: "LC-008"
        type: "Logical"
        level: "LA"
        description: "Brake control unit"
        safety_level: "ASIL_D"
        
        interface_in "Brake Command" {
            protocol: "Internal"
        }
        
        interface_out "Brake Output" {
            protocol: "CAN"
        }
    }
}

trace "LC-003" -> "LC-002" { 
    trace_type: "provides_data"
    rationale: "Radar data to fusion" 
}

trace "LC-004" -> "LC-002" { 
    trace_type: "provides_data"
    rationale: "Camera data to fusion" 
}

trace "LC-005" -> "LC-002" { 
    trace_type: "provides_data"
    rationale: "Lidar data to fusion" 
}

trace "LC-002" -> "LC-006" { 
    trace_type: "provides_data"
    rationale: "Fused data to tracker" 
}

trace "LC-006" -> "LC-007" { 
    trace_type: "provides_data"
    rationale: "Tracking to risk assessment" 
}

trace "LC-007" -> "LC-008" { 
    trace_type: "commands"
    rationale: "Risk assessment to brake control" 
}

trace "LC-001" -> "LC-008" { 
    trace_type: "supervises"
    rationale: "Main control to brake actuation" 
}
