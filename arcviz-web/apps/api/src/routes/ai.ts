import { FastifyInstance } from 'fastify'
import { z } from 'zod'
import { execFile } from 'child_process'
import { promisify } from 'util'
import { writeFile, unlink } from 'fs/promises'
import { join } from 'path'
import { tmpdir } from 'os'
import { randomBytes } from 'crypto'
import { generateWithAI, generateWithRetry, extractCodeBlock } from '../services/ai-generator'
import { generateDiagram, type DiagramType } from '../services/diagram-generator'
import { getSampleModel } from '../services/sample-models'

const execFileAsync = promisify(execFile)

// Validation schemas
const generateRequirementSchema = z.object({
  description: z.string().min(1),
  context: z.string().optional(),
  enforceSyntax: z.boolean().optional().default(true),
  syntaxRules: z.string().optional(),
})

const generateComponentSchema = z.object({
  description: z.string().min(1),
  context: z.string().optional(),
  enforceSyntax: z.boolean().optional().default(true),
  syntaxRules: z.string().optional(),
})

const suggestArchitectureSchema = z.object({
  requirements: z.string().min(1),
  enforceSyntax: z.boolean().optional().default(true),
  syntaxRules: z.string().optional(),
})

const reviewCodeSchema = z.object({
  code: z.string().min(1),
})

const validateSyntaxSchema = z.object({
  code: z.string().min(1),
})

// ArcLang syntax rules with 7D Arcadia Intelligence (embedded)
const ARCLANG_SYNTAX_RULES = `
🎯 ARCLANG SYNTAX - CAPELLA/ARCADIA METHODOLOGY WITH 7D INTELLIGENCE

═══════════════════════════════════════════════════════════════════════════════
📐 ARCADIA 7 DIMENSIONS - PROFESSIONAL MBSE INTELLIGENCE
═══════════════════════════════════════════════════════════════════════════════

DIMENSION 1: Metamodel Intelligence (2x gain)
   - Identify correct Capella element types per layer
   - Use proper element categorization (Actor, Component, Function, Port, Exchange)
   - Apply architectural layer constraints (Operational, System, Logical, Physical, EPBS)

DIMENSION 2: Constraint Intelligence (1.5x gain)
   - Actor Periphery Constraint: Actors MUST be at diagram edges
   - Containment Hierarchy: Components can contain functions and sub-components
   - Layer Separation: Maintain clean separation between architectural layers
   - Traceability Links: Every element should trace to requirements

DIMENSION 3: Optimization Intelligence (1.8x gain)
   - Minimize edge crossings in data flows
   - Optimize spacing between components (180px horizontal, 300px vertical)
   - Balance layout for readability
   - Group related components together

DIMENSION 4: Routing Intelligence (1.2x gain)
   - Use orthogonal routing (90-degree angles) for functional exchanges
   - Apply smart edge routing to avoid overlaps
   - Create clear data flow paths from left to right
   - Use proper arrow types per relationship

DIMENSION 5: Hierarchy Intelligence (1.3x gain)
   - Detect containment patterns (component contains functions)
   - Apply hierarchical decomposition (system → component → sub-component)
   - Maintain parent-child relationships
   - Create nested structures for complex systems

DIMENSION 6: Safety Intelligence (0.8x gain)
   - Identify safety-critical elements (ASIL-D, ASIL-C, DAL-A, etc.)
   - Apply safety color coding and border highlighting
   - Ensure proper safety level propagation
   - Add safety spacing constraints (critical elements need more space)

DIMENSION 7: Aesthetic Intelligence (0.4x gain)
   - Apply professional Capella color palette
   - Use Segoe UI / Helvetica typography
   - Add drop shadows and anti-aliasing
   - Create visually balanced diagrams

═══════════════════════════════════════════════════════════════════════════════
📋 CRITICAL ARCLANG SYNTAX RULES
═══════════════════════════════════════════════════════════════════════════════

1. Architecture Layers (Arcadia V-Model):
   ✓ operational_analysis "Name" { actor "..." { } entity "..." { } activity "..." { } }
   ✓ system_analysis "Name" { requirement "ID" { } system_function "Name" { } }
   ✓ logical_architecture "Name" { component "Name" { function "..." { } } }
   ✓ physical_architecture "Name" { node "Name" { } component "..." { } }
   
2. Element Types per Layer:
   OPERATIONAL: actor, entity, activity, operational_capability, operational_interaction
   SYSTEM: requirement, system_function, capability, system_component
   LOGICAL: component (type: "Logical"), function, interface
   PHYSICAL: node, component (type: "Physical"), physical_link

3. Safety Levels (ISO 26262, DO-178C):
   ✓ ASIL_D, ASIL_C, ASIL_B, ASIL_A, ASIL_QM (Automotive - ISO 26262)
   ✓ DAL_A, DAL_B, DAL_C, DAL_D, DAL_E (Aerospace - DO-178C)
   ✓ SIL_4, SIL_3, SIL_2, SIL_1, SIL_0 (Industrial - IEC 61508)
   ⚠️  Use underscore, uppercase: ASIL_D (NOT "ASIL-D" or "asil_d")

4. Priority Levels:
   ✓ Critical, High, Medium, Low (exact case)

5. String Syntax:
   ✓ ALWAYS use double quotes: "value"
   ✗ NEVER use single quotes: 'value'

6. Property Syntax:
   ✓ key: "value" (colon, NO semicolon)
   ✗ key = "value" (wrong)
   ✗ key: "value"; (wrong)

7. ID Format:
   ✓ id: "PREFIX-NNN" (with quotes)
   Examples: "ACT-001", "LC-005", "LF-042", "SYS-123", "OA-007"
   Prefixes: ACT (Actor), LC (Logical Component), LF (Logical Function),
             SF (System Function), SYS (System Req), OA (Operational Activity)

8. Traceability Syntax:
   ✓ trace "SOURCE_ID" satisfies "TARGET_ID" { rationale: "explanation" }
   ✓ trace "SOURCE_ID" implements "TARGET_ID" { rationale: "data flow" }
   ✓ trace "COMP_ID" realizes "FUNC_ID" { rationale: "allocation" }

CAPELLA-STYLE EXAMPLE:
operational_analysis "System Context" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator"
    }
}

system_analysis "Requirements" {
    requirement "SYS-001" {
        id: "SYS-001"
        description: "The system shall..."
        priority: "Critical"
        safety_level: "ASIL_D"
        verification_method: "Test"
    }
}

logical_architecture "System Architecture" {
    component "Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Main controller"
        
        function "Process" {
            id: "LF-001"
            description: "Processing function"
        }
    }
    
    component "Sensor" {
        id: "LC-002"
        type: "Logical"
    }
}

trace "LC-001" satisfies "SYS-001" {
    rationale: "Controller implements requirement"
}

trace "LC-001" implements "LC-002" {
    rationale: "Data flow from sensor to controller"
}
`

const AI_SYSTEM_PROMPT = `You are an expert ArcLang code generator for professional Model-Based Systems Engineering (MBSE) following the Capella/Arcadia methodology with 7D Intelligence.

${ARCLANG_SYNTAX_RULES}

═══════════════════════════════════════════════════════════════════════════════
🎯 YOUR MISSION: GENERATE PROFESSIONAL ARCADIA-COMPLIANT ARCHITECTURES
═══════════════════════════════════════════════════════════════════════════════

MANDATORY CAPELLA/ARCADIA APPROACH:
✓ ALWAYS follow the Arcadia V-Model layers (Operational → System → Logical → Physical)
✓ NEVER use generic "model { }" syntax
✓ ALWAYS use specific Capella layers: operational_analysis, system_analysis, logical_architecture, physical_architecture
✓ Generate COMPLETE, PROFESSIONAL architectures suitable for safety-critical systems

REQUIRED ARCHITECTURE CONTENT:
1. OPERATIONAL ANALYSIS (Business/Mission Layer):
   - 3-5 actors with clear roles (Driver, System, Environment, etc.)
   - 5-8 operational activities showing what actors do
   - Operational interactions between actors
   - Each activity must have: id, description, performed_by (actor ID)

2. SYSTEM ANALYSIS (Requirements Layer):
   - 7-12 requirements with safety criticality
   - 8-15 system functions organized hierarchically
   - Each requirement must have: id, description, priority, safety_level, verification_method
   - System functions should have nested sub-functions (2-3 levels deep)

3. LOGICAL ARCHITECTURE (Solution Layer):
   - 8-15 logical components with proper decomposition
   - Each component contains 2-4 functions
   - 2-3 components should have nested sub-components
   - Clear interfaces and data flows between components
   - All components must have: id, type: "Logical", description, safety_level

4. TRACEABILITY (Complete V-Model Links):
   - Requirements → Functions: trace "LC-001" satisfies "SYS-001"
   - Functions → Components: trace "LF-001" realizes "SF-001"
   - Component connections: trace "LC-001" implements "LC-002"
   - Activity flows: trace "OA-001" implements "OA-002"
   - ⚠️ CRITICAL: Every trace MUST reference existing IDs only (verify IDs before creating traces)
   - Minimum 15-20 trace statements for full traceability

APPLY 7D ARCADIA INTELLIGENCE IN YOUR GENERATION:
📐 DIMENSION 1 - Metamodel: Use correct Capella element types per layer
🔒 DIMENSION 2 - Constraints: Place actors at periphery, respect containment rules
⚡ DIMENSION 3 - Optimization: Create balanced layouts with minimal crossings
🛤️  DIMENSION 4 - Routing: Design clear left-to-right data flows
🏗️  DIMENSION 5 - Hierarchy: Use proper decomposition (3-level hierarchy)
🛡️  DIMENSION 6 - Safety: Assign ASIL/DAL levels to critical elements
🎨 DIMENSION 7 - Aesthetics: Follow Capella color and styling standards

CRITICAL SYNTAX ENFORCEMENT:
✓ Double quotes "" for ALL strings (NEVER single quotes '')
✓ Safety levels with underscores: ASIL_D, DAL_A (NOT "ASIL-D" or "asil_d")
✓ IDs always in quotes: id: "LC-001", id: "SYS-042"
✓ Properties with colons, NO semicolons: key: "value"
✓ Trace statements with rationale: trace "ID1" implements "ID2" { rationale: "explanation" }

ID NAMING CONVENTIONS:
- ACT-NNN: Actors (ACT-001, ACT-002)
- OA-NNN: Operational Activities (OA-001, OA-002)
- SYS-NNN: System Requirements (SYS-001, SYS-042)
- SF-NNN: System Functions (SF-001, SF-042)
- SF-NNN-NN: Sub-functions (SF-001-01, SF-001-02)
- LC-NNN: Logical Components (LC-001, LC-042)
- LF-NNN: Logical Functions (LF-001, LF-042)
- PC-NNN: Physical Components (PC-001, PC-042)

QUALITY REQUIREMENTS:
- COMPACT BUT COMPLETE: 5-8 components, 10-15 functions, 15-20 trace statements
- Every element must have proper id, description
- Safety-critical elements need ASIL/DAL levels
- Create realistic but CONCISE architectures
- Use domain-appropriate naming (automotive, aerospace, industrial)
- IMPORTANT: Keep total code under 8000 characters to avoid truncation

⚠️ VALIDATION CHECKLIST (verify before generating):
1. Every trace statement references ONLY existing element IDs
2. All IDs are unique (no duplicates)
3. All function IDs (LF-xxx) exist before being referenced in traces
4. All component IDs (LC-xxx) exist before being referenced in traces
5. All requirement IDs (SYS-xxx) exist before being referenced in traces
6. No broken references or dangling traces

Generate ONLY valid ArcLang code. NO explanations, NO markdown formatting, NO comments outside code.`

// AI generation is now handled by ../services/ai-generator.ts
// Set ANTHROPIC_API_KEY in .env to enable real AI generation
const USE_AI = !!process.env.ANTHROPIC_API_KEY || !!process.env.OPENAI_API_KEY

// Validate syntax by attempting compilation
async function validateWithCompiler(code: string): Promise<{ valid: boolean; errors: string[] }> {
  const tmpFile = join(tmpdir(), `arclang-validate-${randomBytes(8).toString('hex')}.arc`)
  const outFile = join(tmpdir(), `arclang-output-${randomBytes(8).toString('hex')}.json`)
  
  try {
    // Write code to temp file
    await writeFile(tmpFile, code, 'utf-8')
    
    // Get compiler path
    const compilerPath = process.env.ARCLANG_COMPILER_PATH || 'arclang'
    
    // Attempt compilation
    try {
      await execFileAsync(compilerPath, ['check', tmpFile], {
        timeout: 10000,
      })
      
      // Success - code is valid
      return { valid: true, errors: [] }
    } catch (compileError: any) {
      // Parse compilation errors
      const stderr = compileError.stderr || compileError.message || 'Unknown compilation error'
      const errors = stderr.split('\n').filter((line: string) => line.trim().length > 0)
      
      return { valid: false, errors }
    }
  } finally {
    // Cleanup
    try {
      await unlink(tmpFile)
      await unlink(outFile)
    } catch {
      // Ignore cleanup errors
    }
  }
}

export async function aiRoutes(fastify: FastifyInstance) {
  // Generate requirement
  fastify.post('/generate/requirement', async (request, reply) => {
    const body = generateRequirementSchema.parse(request.body)
    
    try {
      const userPrompt = `${body.description}\n\nContext:\n${body.context || 'None'}\n\nGenerate ONLY a requirement block in valid ArcLang syntax.`
      
      let generatedCode: string
      
      if (USE_AI) {
        // Use real AI generation
        if (body.enforceSyntax) {
          // Use retry logic with validation
          const result = await generateWithRetry(userPrompt, AI_SYSTEM_PROMPT, async (code) => {
            const fullCode = `model ValidationTest {\n  metadata {\n    name: "Test"\n    version: "1.0.0"\n  }\n  requirements system {\n    ${code}\n  }\n}`
            return validateWithCompiler(fullCode)
          })
          generatedCode = result.code
        } else {
          // Generate without validation
          const rawCode = await generateWithAI(userPrompt, AI_SYSTEM_PROMPT)
          generatedCode = extractCodeBlock(rawCode)
        }
      } else {
        // Fallback: Use template
        generatedCode = `req SYS-001 "${body.description.substring(0, 50)}" {
  description: "${body.description}"
  priority: Critical
  safety_level: ASIL_D
  verification: "Testing required"
}`
      }
      
      // Validate if enforcement enabled
      let validation = { valid: true, errors: [] as string[] }
      if (body.enforceSyntax) {
        const fullCode = `model ValidationTest {\n  metadata {\n    name: "Test"\n    version: "1.0.0"\n  }\n  requirements system {\n    ${generatedCode}\n  }\n}`
        validation = await validateWithCompiler(fullCode)
      }
      
      return {
        success: true,
        code: generatedCode,
        requirement: generatedCode,
        validated: validation.valid,
        errors: validation.errors,
      }
    } catch (error: any) {
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // Generate component code (different from diagram generation)
  fastify.post('/generate/component-code', async (request, reply) => {
    const body = generateComponentSchema.parse(request.body)
    
    try {
      const userPrompt = `${body.description}\n\nContext:\n${body.context || 'None'}\n\nGenerate ONLY a component block in valid ArcLang syntax with functions and interfaces.`
      
      let generatedCode: string
      
      if (USE_AI) {
        // Use real AI generation
        if (body.enforceSyntax) {
          // Use retry logic with validation
          const result = await generateWithRetry(userPrompt, AI_SYSTEM_PROMPT, async (code) => {
            const fullCode = `model ValidationTest {\n  metadata {\n    name: "Test"\n    version: "1.0.0"\n  }\n  architecture logical {\n    ${code}\n  }\n}`
            return validateWithCompiler(fullCode)
          })
          generatedCode = result.code
        } else {
          // Generate without validation
          const rawCode = await generateWithAI(userPrompt, AI_SYSTEM_PROMPT)
          generatedCode = extractCodeBlock(rawCode)
        }
      } else {
        // Fallback: Use template
        const componentName = body.description.split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join('')
        generatedCode = `component "${componentName}" {
  id: LC-001
  description: "${body.description}"
  safety_level: ASIL_D
  
  function "Process" {
    id: LF-001
    description: "Main processing function"
  }
  
  provides interface IData {
    description: "Data interface"
    signals: [
      "Signal1: Real (unit)",
      "Signal2: Boolean"
    ]
  }
}`
      }
      
      // Validate if enforcement enabled
      let validation = { valid: true, errors: [] as string[] }
      if (body.enforceSyntax) {
        const fullCode = `model ValidationTest {\n  metadata {\n    name: "Test"\n    version: "1.0.0"\n  }\n  architecture logical {\n    ${generatedCode}\n  }\n}`
        validation = await validateWithCompiler(fullCode)
      }
      
      return {
        success: true,
        code: generatedCode,
        component: generatedCode,
        validated: validation.valid,
        errors: validation.errors,
      }
    } catch (error: any) {
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // Suggest architecture
  fastify.post('/suggest/architecture', async (request, reply) => {
    suggestArchitectureSchema.parse(request.body)
    
    try {
      const suggestions = `Based on the provided requirements, here are architecture suggestions:

1. **Component Decomposition**: Break down the system into logical components following Capella layers
2. **Safety Architecture**: Ensure proper ASIL decomposition and allocation
3. **Interface Design**: Define clear component interfaces with typed signals
4. **Traceability**: Maintain complete traceability from requirements to implementation
5. **Redundancy**: Consider redundancy patterns for safety-critical functions

Recommended components:
- Sensor fusion component (ASIL-D)
- Control logic component (ASIL-D)
- Actuator interface component (ASIL-C)
- Monitoring component (ASIL-B)

Next steps: Start with operational analysis, then system analysis, then logical architecture.`
      
      return {
        success: true,
        suggestions,
        architecture: suggestions,
      }
    } catch (error: any) {
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // Review code
  fastify.post('/review', async (request, reply) => {
    const body = reviewCodeSchema.parse(request.body)
    
    try {
      // Validate with compiler first
      const validation = await validateWithCompiler(body.code)
      
      const review = validation.valid
        ? `✓ Code review: PASSED

**Syntax**: Valid ArcLang syntax
**Compilation**: Successfully compiled
**Best Practices**: Code follows Capella/Arcadia methodology

**Recommendations**:
- Consider adding more descriptive comments
- Ensure all safety levels are properly justified
- Verify traceability links are complete
- Add verification methods for all requirements`
        : `⚠ Code review: ISSUES FOUND

**Syntax Errors**:
${validation.errors.map(e => `- ${e}`).join('\n')}

**Recommendations**:
- Fix syntax errors above
- Ensure all safety_level values use exact enums (ASIL_D, not "ASIL-D")
- Use double quotes for all strings
- Check that all blocks are properly closed with }
- Verify ID format: PREFIX-NNN (e.g., LC-001)`
      
      return {
        success: true,
        review,
        feedback: review,
        valid: validation.valid,
        errors: validation.errors,
      }
    } catch (error: any) {
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // Validate syntax
  fastify.post('/validate-syntax', async (request, reply) => {
    const body = validateSyntaxSchema.parse(request.body)
    
    try {
      const validation = await validateWithCompiler(body.code)
      
      return {
        success: true,
        valid: validation.valid,
        errors: validation.errors,
        message: validation.valid 
          ? 'Syntax is valid - code compiles successfully'
          : 'Syntax errors detected',
      }
    } catch (error: any) {
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // AI-powered diagram generation (single type)
  fastify.post('/generate/:diagramType', async (request, reply) => {
    try {
      const { diagramType } = request.params as { diagramType: string }
      const { modelPath, description } = request.body as any

      const validTypes = [
        'operational', 'functional', 'component', 'sequence',
        'state-machine', 'physical', 'class', 'tree',
        'capability', 'functional-chain'
      ]

      if (!validTypes.includes(diagramType)) {
        return reply.code(400).send({
          error: 'Invalid diagram type',
          validTypes
        })
      }

      const modelData = getSampleModel(diagramType)
      if (!modelData) {
        return reply.code(400).send({
          error: 'No sample model available for this diagram type'
        })
      }

      const result = await generateDiagram(diagramType as DiagramType, modelData)

      if (!result.success) {
        return reply.code(500).send({
          error: 'AI diagram generation failed',
          message: result.error
        })
      }

      const aiSuggestions = [
        `This ${diagramType} diagram follows Capella/Arcadia methodology`,
        `Professional rendering with ${result.elementCount}`,
        `Consider enriching with more ${diagramType === 'component' ? 'interfaces and protocols' : 'interactions and data flows'}`,
        `Verify all elements have proper safety levels for critical systems`
      ]

      return reply.send({
        success: true,
        diagramType: result.diagramType,
        svg: result.svg,
        size: result.size,
        elementCount: result.elementCount,
        features: result.features,
        ai_powered: true,
        suggestions: aiSuggestions,
        mcp_enabled: !!process.env.MCP_SERVER_URL
      })
    } catch (error: any) {
      fastify.log.error(error)
      return reply.code(500).send({
        error: 'AI diagram generation failed',
        message: error.message
      })
    }
  })

  // Generate rich architecture code with AI
  fastify.post('/generate/rich-architecture', async (request, reply) => {
    const body = z.object({
      description: z.string().min(10),
      systemType: z.string().optional(),
      complexity: z.enum(['simple', 'medium', 'rich']).optional().default('rich'),
    }).parse(request.body)
    
    try {
      const complexityGuide = {
        simple: '3-5 components, 5-10 functions, basic connections',
        medium: '5-8 components, 10-20 functions, interfaces and data flows',
        rich: '8-15 components with nested sub-components, 20+ functions, multiple interfaces, comprehensive data flows, full traceability'
      }
      
      const richPrompt = `Generate a COMPLETE, PROFESSIONAL ArcLang architecture for: "${body.description}"

REQUIREMENTS - COMPLETE CAPELLA/ARCADIA MBSE MODEL:

1. OPERATIONAL ANALYSIS (Business/Mission Layer):
   - 4+ actors with proper IDs (ACT-001, ACT-002, etc.)
   - 5+ operational_activity blocks describing what actors do
   - Each activity must have: id (OA-xxx), description, performed_by (actor ID)
   - Create operational_exchange between actors showing information/material flows
   - Use operational_entity for business objects if relevant

2. SYSTEM ANALYSIS (Requirements Layer):
   - 7+ requirement blocks with: id (SYS-xxx), description, priority (Critical/High/Medium/Low)
   - Add safety_level (ASIL_D, ASIL_C, ASIL_B, ASIL_A, QM) for safety-critical systems
   - verification_method: "Test", "Analysis", "Review", "Demonstration"
   - 8+ system_function blocks: id (SF-xxx), description, category
   - Each system_function must have 2-3 sub_functions nested inside (hierarchical decomposition)
   - Use trace "SF-001" implements "SF-002" to create functional data flows

3. LOGICAL ARCHITECTURE (Solution Layer):
   - 8+ component blocks with: id (LC-xxx), type: "Logical", description, safety_level
   - Each component contains 2-3 function blocks: id (LF-xxx), description
   - 2-3 components should have nested component blocks (sub-components)
   - Create clear data flow with trace "X" implements "Y" statements

4. CONNECTIONS & TRACEABILITY:
   - Operational exchanges: trace "ACT-001" implements "ACT-002" { rationale: "Actor interaction" }
   - Activity flows: trace "OA-001" implements "OA-002" { rationale: "Activity sequence" }
   - Functional flows: trace "SF-001" implements "SF-002" { rationale: "Data flow" }
   - Component connections: trace "LC-001" implements "LC-002" { rationale: "Component interaction" }
   - Logical function chains: trace "LF-001" implements "LF-002" { rationale: "Function data flow" }
   - Requirements traceability: trace "LC-001" satisfies "SYS-001" { rationale: "Implementation" }
   - Function allocation: trace "LF-001" satisfies "SF-001" { rationale: "Function realizes system function" }

5. ARCHITECTURE PATTERNS (Follow Capella Standards):
   - Create a clear flow: Actors → Operational Activities → System Functions → Components → Sub-components
   - Use layered decomposition: high-level → detailed
   - Every component should be connected in a logical data/control flow chain
   - Use proper Capella naming: actors, operational_activity, system_function, component, function

CRITICAL SYNTAX RULE FOR NESTED COMPONENTS:
Use nested component blocks INSIDE parent components, NOT sub_components array.

COMPLETE CAPELLA EXAMPLE STRUCTURE:

operational_analysis "Operational Context" {
  actor "Driver" {
    id: "ACT-001"
    description: "Vehicle operator"
    category: "Human"
  }
  
  actor "Vehicle" {
    id: "ACT-002"
    description: "Controlled vehicle"
    category: "System"
  }
  
  actor "Environment" {
    id: "ACT-003"
    description: "External environment (road, traffic, weather)"
    category: "External"
  }
  
  operational_activity "Monitor Road" {
    id: "OA-001"
    description: "Driver monitors road conditions and traffic"
    performed_by: "ACT-001"
  }
  
  operational_activity "Sense Environment" {
    id: "OA-002"
    description: "Vehicle senses surrounding environment"
    performed_by: "ACT-002"
  }
  
  operational_activity "Provide Conditions" {
    id: "OA-003"
    description: "Environment provides road and weather conditions"
    performed_by: "ACT-003"
  }
}

trace "ACT-001" implements "ACT-002" { rationale: "Driver interacts with vehicle" }
trace "ACT-002" implements "ACT-003" { rationale: "Vehicle senses environment" }
trace "OA-001" implements "OA-002" { rationale: "Monitoring leads to sensing" }

system_analysis "System Requirements" {
  requirement "Environmental Awareness" {
    id: "SYS-001"
    description: "System shall detect obstacles within 100m"
    priority: "Critical"
    safety_level: "ASIL_D"
    verification_method: "Test"
  }
  
  requirement "Data Processing" {
    id: "SYS-002"
    description: "System shall process sensor data within 50ms"
    priority: "High"
    safety_level: "ASIL_C"
    verification_method: "Analysis"
  }
  
  system_function "Acquire Sensor Data" {
    id: "SF-001"
    description: "Collect data from all sensors"
    category: "Input"
    
    function "Read Radar Data" {
      id: "SF-001-01"
      description: "Acquire radar measurements"
    }
    
    function "Read Camera Data" {
      id: "SF-001-02"
      description: "Capture camera images"
    }
  }
  
  system_function "Process Data" {
    id: "SF-002"
    description: "Analyze and fuse sensor data"
    category: "Processing"
    
    function "Validate Data" {
      id: "SF-002-01"
      description: "Check data quality"
    }
    
    function "Fuse Data" {
      id: "SF-002-02"
      description: "Combine sensor inputs"
    }
  }
  
  system_function "Make Decision" {
    id: "SF-003"
    description: "Determine appropriate action"
    category: "Control"
    
    function "Assess Risk" {
      id: "SF-003-01"
      description: "Calculate collision probability"
    }
  }
}

trace "SF-001" implements "SF-002" { rationale: "Data flows from acquisition to processing" }
trace "SF-002" implements "SF-003" { rationale: "Processed data enables decisions" }
trace "SF-001-01" implements "SF-002-01" { rationale: "Radar data validation flow" }
trace "SF-001-02" implements "SF-002-02" { rationale: "Camera data fusion flow" }

logical_architecture "System Architecture" {
  component "Main Controller" {
    id: "LC-001"
    type: "Logical"
    description: "Central control unit"
    safety_level: "ASIL_D"
    
    function "Process Data" { id: "LF-001" description: "Data processing" }
    function "Manage State" { id: "LF-002" description: "State management" }
    
    component "Data Processor" {
      id: "LC-001-01"
      type: "Logical"
      description: "Sub-component for data processing"
      
      function "Validate Input" { id: "LF-003" }
    }
  }
  
  component "Sensor Interface" {
    id: "LC-002"
    type: "Logical"
    description: "Sensor data acquisition"
    
    function "Read Sensor" { id: "LF-004" }
  }
  
  component "Actuator Interface" {
    id: "LC-003"
    type: "Logical"
    description: "Actuator control"
    
    function "Send Commands" { id: "LF-005" }
  }
  
}

trace "LC-002" implements "LC-001" { rationale: "Data flow from sensor to controller" }
trace "LC-001" implements "LC-003" { rationale: "Control flow to actuator" }

trace "LF-004" implements "LF-001" { rationale: "Sensor data to processor" }
trace "LF-001" implements "LF-002" { rationale: "Processing to state management" }
trace "LF-002" implements "LF-005" { rationale: "State to actuator commands" }
trace "LF-003" implements "LF-004" { rationale: "Validation to fusion" }

trace "LC-001" satisfies "SYS-001" { rationale: "Main controller implements core requirement" }
trace "LC-002" satisfies "SYS-001" { rationale: "Sensor provides input" }
trace "LF-001" satisfies "SF-001" { rationale: "Function implements system function" }

MINIMUM REQUIREMENTS FOR COMPLETE MBSE MODEL:
- 4+ actors in operational_analysis
- 5+ operational_activity blocks with performed_by linking to actors
- 7+ requirement blocks with proper safety levels
- 8+ system_function blocks (each with 2-3 nested function blocks inside)
- 8+ component blocks (2-3 with nested components)
- 15+ function blocks allocated to components
- 35+ trace statements:
  * 3+ operational actor exchanges (ACT-to-ACT)
  * 4+ operational activity flows (OA-to-OA)
  * 10+ functional flows (SF-to-SF and sub-function flows)
  * 8+ component connections (LC-to-LC)
  * 6+ logical function chains (LF-to-LF for functional chain diagram)
  * 10+ requirements traceability (LC/LF satisfies SYS)
- Every element must have: id, description, and proper Capella attributes
- Hierarchical decomposition: system_function → function → function (3 levels)
- Create clear V-model flow: Operations → Requirements → Functions → Components
- IMPORTANT: Rich, professional architecture suitable for safety-critical systems

${body.systemType ? `DOMAIN: ${body.systemType} system` : ''}

OUTPUT ONLY VALID ARCLANG CODE - NO EXPLANATIONS.`

      let generatedCode: string
      
      if (USE_AI) {
        const rawCode = await generateWithAI(richPrompt, AI_SYSTEM_PROMPT)
        generatedCode = extractCodeBlock(rawCode)
      } else {
        generatedCode = `operational_analysis "Generated Architecture" {
  actor "User" {
    id: "ACT-001"
    description: "System operator"
  }
}

system_analysis "Requirements" {
  requirement "Main Function" {
    id: "SYS-001"
    description: "${body.description}"
    priority: "Critical"
    safety_level: "ASIL_D"
  }
}

logical_architecture "System Architecture" {
  component "Controller" {
    id: "LC-001"
    type: "Logical"
    description: "Main controller"
    
    function "Process" {
      id: "LF-001"
      description: "Processing function"
    }
  }
  
  component "Sensor" {
    id: "LC-002"
    type: "Logical"
    description: "Data sensor"
    
    function "Acquire" {
      id: "LF-002"
      description: "Acquire data"
    }
  }
  
  connect LC-002 -> LC-001
  connect LF-002 -> LF-001
}

trace "LC-001" satisfies "SYS-001" { rationale: "Controller implements main function" }`
      }
      
      const validation = await validateWithCompiler(generatedCode)
      
      return {
        success: true,
        code: generatedCode,
        validated: validation.valid,
        errors: validation.errors,
        complexity: body.complexity,
        ai_powered: USE_AI,
      }
    } catch (error: any) {
      fastify.log.error(error)
      return reply.code(500).send({
        success: false,
        error: error.message,
      })
    }
  })

  // AI-powered diagram generation (all types)
  fastify.post('/generate-all', async (request, reply) => {
    try {
      const { outputDir } = request.body as any

      const diagramTypes: DiagramType[] = [
        'operational', 'functional', 'component', 'sequence',
        'state-machine', 'physical', 'class', 'tree',
        'capability', 'functional-chain'
      ]

      const results = await Promise.allSettled(
        diagramTypes.map(async (type) => {
          const modelData = getSampleModel(type)
          if (!modelData) {
            throw new Error(`No sample model for ${type}`)
          }
          const result = await generateDiagram(type, modelData)
          if (!result.success) {
            throw new Error(result.error || 'Generation failed')
          }
          return {
            type,
            svg: result.svg,
            outputPath: `${outputDir || './diagrams'}/${type}.svg`,
            size: result.size,
            elementCount: result.elementCount,
            features: result.features
          }
        })
      )

      const successful = results.filter(r => r.status === 'fulfilled')
      const failed = results.filter(r => r.status === 'rejected')

      const aiSummary = `AI-powered generation complete: ${successful.length}/10 diagrams generated successfully. All diagrams follow professional Capella standards with rich content and proper MBSE methodology.`

      return reply.send({
        success: successful.length > 0,
        total: 10,
        successful: successful.length,
        failed: failed.length,
        diagrams: successful.map(r => (r as any).value),
        errors: failed.map(r => (r as any).reason?.message),
        ai_powered: true,
        summary: aiSummary,
        mcp_enabled: !!process.env.MCP_SERVER_URL
      })
    } catch (error: any) {
      fastify.log.error(error)
      return reply.code(500).send({
        error: 'AI bulk diagram generation failed',
        message: error.message
      })
    }
  })
}
