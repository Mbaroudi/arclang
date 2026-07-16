// ArcLang Syntax Schema - Strict validation rules for AI generation
// Based on Capella/Arcadia MBSE methodology

export const ARCLANG_SYNTAX_RULES = `
# ARCLANG SYNTAX RULES - MANDATORY FOR ALL GENERATED CODE

## CRITICAL RULES (MUST FOLLOW):

1. **Model Structure**:
   - Always start with: model <Name> { }
   - Use PascalCase for model names
   - Model body must contain metadata block first

2. **Metadata Block** (REQUIRED):
   metadata {
     name: "String"
     version: "X.Y.Z"
     author: "Name"
     description: "Text"
     safety_standard: "iso26262" | "do178c" | "iec61508"
   }

3. **Requirements Syntax**:
   requirements <level> {
     req <ID> "Name" {
       description: "Text"
       priority: Critical | High | Medium | Low
       safety_level: ASIL_D | ASIL_C | ASIL_B | ASIL_A | DAL_A | DAL_B | DAL_C | DAL_D | QM
       traces: [REQ-IDs]
       verification: "Method"
       rationale: "Text"
     }
   }
   
   Valid levels: stakeholder, system, logical, physical

4. **Architecture Layers** (ONE OF):
   - architecture operational { }  // OA - Operational Analysis
   - architecture system { }       // SA - System Analysis  
   - architecture logical { }      // LA - Logical Architecture
   - architecture physical { }     // PA - Physical Architecture
   - architecture epbs { }         // EPBS

5. **Component Syntax**:
   component "Name" {
     id: LC-001
     description: "Text"
     safety_level: ASIL_D | ASIL_C | ASIL_B | ASIL_A
     component_type: "Logical" | "Physical" | "System"
     
     function "FunctionName" {
       id: LF-001
       description: "Text"
     }
     
     provides interface IName {
       description: "Text"
       signals: [
         "SignalName: Type (unit)",
         "Speed: Real (m/s)"
       ]
     }
   }

6. **Connections**:
   connect Source.IInterface -> Target
   connect Sensor.ISensorData -> Controller

7. **Exchanges**:
   exchange "Name" {
     from: ComponentA
     to: ComponentB
     type: data | control | event
     protocol: "CAN" | "Ethernet" | "LIN"
   }

8. **Traceability**:
   traceability {
     trace STK-001 -> [SYS-001, SYS-002]
     trace SYS-001 -> [ComponentName]
   }

9. **Safety Levels** (EXACT VALUES ONLY):
   - ASIL_D, ASIL_C, ASIL_B, ASIL_A (Automotive ISO 26262)
   - DAL_A, DAL_B, DAL_C, DAL_D (Aerospace DO-178C)
   - QM (Quality Managed)

10. **Actors** (Operational Architecture):
    actor "Name" {
      description: "Text"
      role: "Role description"
    }

11. **Physical Nodes**:
    node "ECU_Name" {
      description: "Text"
      allocates: [LogicalComponent1, LogicalComponent2]
      hardware: "Hardware specs"
    }

## FORBIDDEN PATTERNS:

❌ NO semicolons at end of blocks
❌ NO single quotes (use double quotes)
❌ NO undefined safety levels (use exact values above)
❌ NO missing colons after property names
❌ NO camelCase for keywords (use lowercase)
❌ NO missing brackets { }
❌ NO undefined priority values
❌ NO invalid layer names

## CORRECT EXAMPLES:

### Example 1: Complete System
\`\`\`arclang
model AutonomousDrivingSystem {
  metadata {
    name: "Autonomous Driving System"
    version: "2.0.0"
    author: "Safety Engineer"
    description: "ASIL-D autonomous driving architecture"
    safety_standard: "iso26262"
  }
  
  requirements system {
    req SYS-001 "Vehicle Control" {
      description: "System shall control vehicle speed and direction"
      priority: Critical
      safety_level: ASIL_D
      verification: "Hardware-in-loop testing"
      rationale: "Core safety requirement"
    }
  }
  
  architecture logical {
    component "PerceptionController" {
      id: LC-001
      description: "Processes sensor data"
      safety_level: ASIL_D
      
      function "ProcessCameraData" {
        id: LF-001
        description: "Camera image processing"
      }
      
      provides interface ISensorFusion {
        description: "Fused sensor output"
        signals: [
          "ObjectList: Array",
          "Speed: Real (m/s)"
        ]
      }
    }
    
    component "PathPlanner" {
      id: LC-002
      description: "Plans vehicle trajectory"
      safety_level: ASIL_D
      
      function "CalculateTrajectory" {
        id: LF-002
        description: "Compute optimal path"
      }
    }
    
    connect PerceptionController.ISensorFusion -> PathPlanner
  }
  
  traceability {
    trace SYS-001 -> [PerceptionController, PathPlanner]
  }
}
\`\`\`

### Example 2: Requirements Only
\`\`\`arclang
model RequirementsSpec {
  metadata {
    name: "System Requirements"
    version: "1.0.0"
  }
  
  requirements stakeholder {
    req STK-001 "Safe Operations" {
      description: "System shall operate safely"
      priority: Critical
      safety_level: ASIL_D
    }
  }
  
  requirements system {
    req SYS-001 "Speed Control" {
      description: "Control vehicle speed"
      priority: Critical
      safety_level: ASIL_D
      traces: [STK-001]
    }
  }
}
\`\`\`

### Example 3: Physical Architecture
\`\`\`arclang
model PhysicalArchitecture {
  metadata {
    name: "ECU Allocation"
    version: "1.0.0"
  }
  
  architecture physical {
    node "CentralECU" {
      description: "Main control unit"
      allocates: [PerceptionController, PathPlanner]
      hardware: "ARM Cortex-A72, 4GB RAM"
    }
    
    node "SensorECU" {
      description: "Sensor interface unit"
      allocates: [SensorDriver]
      hardware: "ARM Cortex-M7, 256MB RAM"
    }
  }
}
\`\`\`

## VALIDATION CHECKLIST:

✓ Model starts with 'model' keyword
✓ Metadata block present and complete
✓ All strings use double quotes
✓ All property assignments use colon (:)
✓ Safety levels use exact enum values
✓ IDs follow pattern: STK-XXX, SYS-XXX, LC-XXX, LF-XXX
✓ All blocks properly closed with }
✓ Trace references match existing IDs
✓ Component connections reference valid interfaces
✓ No syntax errors (missing brackets, quotes, colons)

## AI GENERATION INSTRUCTIONS:

When generating ArcLang code:
1. ALWAYS start with model + metadata
2. Use EXACT safety level values (ASIL_D, not "ASIL-D" or "asil_d")
3. Use EXACT priority values (Critical, High, Medium, Low)
4. Follow ID naming conventions strictly
5. Always use double quotes for strings
6. Always include description fields
7. Validate all references (traces, connections)
8. Close all blocks properly
9. Use proper indentation (2 or 4 spaces)
10. Test generated code mentally before outputting

REMEMBER: Code must compile without errors on first try!
`

export const AI_GENERATION_PROMPT_PREFIX = `
You are an expert ArcLang code generator. ArcLang is a domain-specific language for Model-Based Systems Engineering following the Capella/Arcadia methodology.

CRITICAL: Your generated code MUST be syntactically perfect and compile without errors.

Follow these MANDATORY rules:

${ARCLANG_SYNTAX_RULES}

IMPORTANT:
- Double-check all syntax before responding
- Validate all IDs and references
- Use exact enum values for safety_level and priority
- Test your output mentally
- DO NOT invent new keywords or syntax
- DO NOT use patterns not shown in examples above
`

export const VALIDATION_PROMPT = `
Validate this ArcLang code against the syntax rules:

{CODE}

Check for:
1. Correct model structure
2. Proper metadata block
3. Valid safety levels (ASIL_X, DAL_X, QM)
4. Valid priority values (Critical, High, Medium, Low)
5. Proper quote usage (double quotes only)
6. Correct property syntax (name: value)
7. Valid ID patterns
8. Proper block closures
9. Valid trace references

Return:
- "VALID" if code is correct
- List of specific errors if invalid
`

export function validateArcLangSyntax(code: string): { valid: boolean; errors: string[] } {
  const errors: string[] = []
  
  // Detect if this is a partial block (requirements, architecture, component only)
  const isPartialBlock = code.trim().match(/^(requirements|architecture|component|function|interface|actor|exchange|traceability)\s/m) && !code.includes('model ')
  
  // Skip model/metadata checks for partial blocks
  if (!isPartialBlock) {
    // Check model declaration
    if (!code.match(/^model\s+[A-Z][A-Za-z0-9]*\s*\{/m)) {
      errors.push('Missing or invalid model declaration. Must start with: model <PascalCaseName> {')
    }

    // Check metadata block
    if (!code.includes('metadata {')) {
      errors.push('Missing metadata block. Every model must have metadata.')
    }
  }

  // Check for single quotes (forbidden)
  if (code.includes("'")) {
    errors.push('Single quotes not allowed. Use double quotes for all strings.')
  }

  // Check for semicolons at end of lines (forbidden in blocks)
  if (code.match(/:\s*"[^"]*"\s*;/)) {
    errors.push('Remove semicolons after property values. Syntax is: name: "value" (no semicolon)')
  }

  // Check safety level format
  const invalidSafetyLevels = code.match(/safety_level:\s*"(ASIL-[A-D]|DAL-[A-D]|asil_[a-d]|dal_[a-d])"/g)
  if (invalidSafetyLevels) {
    errors.push(`Invalid safety level format. Use: ASIL_D, ASIL_C, ASIL_B, ASIL_A, DAL_A, DAL_B, DAL_C, DAL_D, or QM (underscore, not hyphen, uppercase)`)
  }

  // Check for balanced braces
  const openBraces = (code.match(/{/g) || []).length
  const closeBraces = (code.match(/}/g) || []).length
  if (openBraces !== closeBraces) {
    errors.push(`Unbalanced braces: ${openBraces} opening, ${closeBraces} closing`)
  }

  // Check for invalid priority values (case-sensitive)
  const priorityMatches = code.match(/priority:\s*([A-Za-z]+)/g)
  if (priorityMatches) {
    const validPriorities = ['Critical', 'High', 'Medium', 'Low']
    priorityMatches.forEach(match => {
      const value = match.replace(/priority:\s*/, '')
      if (!validPriorities.includes(value)) {
        errors.push(`Priority must be: Critical, High, Medium, or Low (exact case). Found: ${value}`)
      }
    })
  }

  // Check for proper ID format (PREFIX-NNN where PREFIX is uppercase letters)
  const idMatches = code.match(/id:\s*([A-Za-z0-9-]+)/g)
  if (idMatches) {
    idMatches.forEach(match => {
      const id = match.replace(/id:\s*/, '').trim()
      // Valid format: 2+ uppercase letters, hyphen, 1+ digits (e.g., STK-001, SYS-1, LC-01)
      if (!id.match(/^[A-Z]{2,}-\d+$/)) {
        errors.push(`ID "${id}" must follow pattern: PREFIX-NNN (e.g., STK-001, SYS-001, LC-001)`)
      }
    })
  }

  return {
    valid: errors.length === 0,
    errors,
  }
}

export const SYNTAX_EXAMPLES = {
  requirement: `req SYS-001 "System Requirement" {
  description: "The system shall process data"
  priority: Critical
  safety_level: ASIL_D
  verification: "Unit testing"
}`,
  
  component: `component "DataProcessor" {
  id: LC-001
  description: "Processes sensor data"
  safety_level: ASIL_C
  
  function "ProcessData" {
    id: LF-001
    description: "Data processing function"
  }
}`,
  
  architecture: `architecture logical {
  component "Controller" {
    id: LC-001
    description: "Main controller"
    safety_level: ASIL_D
  }
  
  component "Sensor" {
    id: LC-002
    description: "Sensor interface"
    safety_level: ASIL_B
  }
  
  connect Sensor.IData -> Controller
}`,

  complete: `model ExampleSystem {
  metadata {
    name: "Example System"
    version: "1.0.0"
    author: "Engineer"
    safety_standard: "iso26262"
  }
  
  requirements system {
    req SYS-001 "Core Requirement" {
      description: "System shall function safely"
      priority: Critical
      safety_level: ASIL_D
    }
  }
  
  architecture logical {
    component "MainComponent" {
      id: LC-001
      description: "Main system component"
      safety_level: ASIL_D
      
      function "MainFunction" {
        id: LF-001
        description: "Primary function"
      }
    }
  }
  
  traceability {
    trace SYS-001 -> [MainComponent]
  }
}`,
}
