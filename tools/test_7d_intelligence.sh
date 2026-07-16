#!/bin/bash

echo "=== Testing 7D Intelligence Integration ==="
echo ""

# Create test arclang file
cat > /tmp/test_7d.arc << 'EOF'
architecture "7D Intelligence Test" {
    
    operational_analysis "Test OA" {
        actor "Driver" {
            type: OperationalActor
            description: "Human driver"
        }
        
        entity "Vehicle" {
            type: OperationalEntity
            description: "The vehicle system"
        }
        
        activity "Monitor Environment" {
            description: "Monitor surroundings"
        }
    }
    
    logical_architecture "Test LA" {
        component "Sensor Module" {
            type: LogicalComponent
            safety_level: "ASIL-D"
            
            function "Acquire Data" {
                description: "Acquire sensor data"
            }
        }
        
        component "Processing Unit" {
            type: LogicalComponent
            asil: "ASIL-C"
            
            function "Process Data" {
                description: "Process incoming data"
            }
        }
        
        interface "Sensor Data" from "Sensor Module" to "Processing Unit"
    }
    
    trace "Monitor Environment" -> "Acquire Data" {
        type: "satisfies"
    }
}
EOF

echo "Compiling with Rust compiler to see 7D Intelligence logs..."
echo ""

cd /Users/malek/arclang

# Use the Rust library directly via a simple Rust program
cat > /tmp/test_7d.rs << 'RUST'
use std::fs;

fn main() {
    let code = fs::read_to_string("/tmp/test_7d.arc").expect("Failed to read file");
    
    // Import arclang
    // This is pseudocode - we'll use CLI instead
    println!("Code loaded: {} bytes", code.len());
}
RUST

# Instead, let's trace the build command
echo "Running diagram generation..."
./target/release/arclang diagram /tmp/test_7d.arc --output /tmp/test_7d_output.html --format operational 2>&1 | grep -A 1 -B 1 "7D\|DIMENSION"

echo ""
echo "=== Test Complete ==="
