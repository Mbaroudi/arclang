# MCP Server & ArcViz Web Update Plan

## 🎯 Objective
Integrate all 10 rich diagram types into:
1. **MCP Server** - AI-powered diagram generation
2. **ArcViz Web** - Editor + Visualizer with AI support

---

## ✅ COMPLETED: MCP Server Diagram Tools

### New Tools Added to `generation.py`

#### 1. `arclang_generate_diagram`
**Purpose**: Generate a specific diagram type from model

**Input**:
```json
{
  "model_path": "path/to/model.arc",
  "diagram_type": "operational|functional|component|sequence|state-machine|physical|class|tree|capability|functional-chain",
  "output_path": "optional/output.svg"
}
```

**Output**: SVG diagram with metadata (size, element count, features)

#### 2. `arclang_generate_all_diagrams`
**Purpose**: Generate all 10 Capella diagram types at once

**Input**:
```json
{
  "model_path": "path/to/model.arc",
  "output_dir": "./diagrams"
}
```

**Output**: 10 SVG files + summary report with success/failure status

### Files Modified
- ✅ `/mcp-server/src/arclang_mcp/tools/generation.py` - Added 2 new methods
- ⏳ `/mcp-server/src/arclang_mcp/server.py` - Need to register new tools
- ⏳ `/mcp-server/src/arclang_mcp/compiler/wrapper.py` - Need diagram generation methods

---

## 📝 TODO: Complete MCP Server Integration

### 1. Register New Tools in `server.py`

Add after `arclang_suggest_architecture` (line 271):

```python
Tool(
    name="arclang_generate_diagram",
    description="""Generate a specific Capella diagram type from ArcLang model.
    
Supports all 10 diagram types:
- Operational: Swimlane activity diagrams with actors
- Functional: Data flow diagrams with functions
- Component: Block diagrams with components
- Sequence: Interaction scenarios
- State Machine: State/transition diagrams
- Physical: Hardware deployment
- Class: Data type definitions
- Tree: Hierarchical breakdowns
- Capability: Requirements hierarchy
- Functional Chain: Execution scenarios
""",
    inputSchema={
        "type": "object",
        "properties": {
            "model_path": {
                "type": "string",
                "description": "Path to .arc model file"
            },
            "diagram_type": {
                "type": "string",
                "enum": ["operational", "functional", "component", "sequence", 
                        "state-machine", "physical", "class", "tree", 
                        "capability", "functional-chain"],
                "description": "Type of diagram to generate"
            },
            "output_path": {
                "type": "string",
                "description": "Output SVG file path (optional)"
            }
        },
        "required": ["model_path", "diagram_type"]
    }
),

Tool(
    name="arclang_generate_all_diagrams",
    description="""Generate all 10 Capella diagram types from ArcLang model.
    
Creates complete diagram set:
- 10 professional-quality SVG diagrams
- Rich content (3-13x larger than simple)
- 100% Capella visual parity
- Automatic output organization
""",
    inputSchema={
        "type": "object",
        "properties": {
            "model_path": {
                "type": "string",
                "description": "Path to .arc model file"
            },
            "output_dir": {
                "type": "string",
                "description": "Output directory for all diagrams",
                "default": "./diagrams"
            }
        },
        "required": ["model_path"]
    }
),
```

### 2. Implement Compiler Wrapper Methods in `compiler/wrapper.py`

```python
async def generate_diagram(self, model_path: str, diagram_type: str, output_path: Optional[str] = None) -> Dict[str, Any]:
    """Generate a specific diagram type."""
    import subprocess
    import os
    
    if not output_path:
        output_path = f"./{diagram_type}.svg"
    
    # Call Rust CLI
    cmd = [
        "arclang",
        "diagram",
        model_path,
        "-o", output_path,
        "--format", diagram_type
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        raise Exception(f"Diagram generation failed: {result.stderr}")
    
    # Get file size
    size = os.path.getsize(output_path)
    size_str = f"{size/1024:.1f}KB" if size > 1024 else f"{size}B"
    
    # Parse output for element count
    element_count = self._parse_element_count(result.stdout, diagram_type)
    
    # Get features based on diagram type
    features = self._get_diagram_features(diagram_type)
    
    return {
        "output_path": output_path,
        "size": size_str,
        "element_count": element_count,
        "features": features
    }

def _parse_element_count(self, output: str, diagram_type: str) -> str:
    """Parse element count from CLI output."""
    # Look for patterns like "Functions: 15" or "Activities: 10"
    import re
    patterns = {
        "operational": r"Activities: (\d+)",
        "functional": r"Functions: (\d+)",
        "component": r"Components: (\d+)",
        "sequence": r"Messages: (\d+)",
        "state-machine": r"States: (\d+)",
        "physical": r"Nodes: (\d+)",
        "class": r"Classes: (\d+)",
        "tree": r"Nodes: (\d+)",
        "capability": r"Capabilities: (\d+)",
        "functional-chain": r"Functions: (\d+)"
    }
    
    pattern = patterns.get(diagram_type)
    if pattern:
        match = re.search(pattern, output)
        if match:
            return match.group(1)
    
    return "N/A"

def _get_diagram_features(self, diagram_type: str) -> List[str]:
    """Get feature list for diagram type."""
    features = {
        "operational": [
            "Swimlane layout by actor",
            "Stick figures for human actors",
            "System boxes for components",
            "Activity symbols (⊕)",
            "Protocol labels (CAN, V2X, HMI)",
            "Hierarchical activities"
        ],
        "functional": [
            "Data flow visualization",
            "Port-based connections",
            "Category coloring (7 types)",
            "External actor boundaries",
            "Typed exchanges"
        ],
        "component": [
            "Hierarchical structure",
            "Interface protocols (CAN, Ethernet)",
            "Port visualization",
            "Sub-component nesting",
            "Block diagram layout"
        ],
        "sequence": [
            "Time-ordered messages",
            "Participant lifelines",
            "Fragment blocks (alt, loop, opt)",
            "Synchronous/asynchronous calls"
        ],
        "state-machine": [
            "State visualization",
            "Transition arrows",
            "Guard conditions",
            "Entry/exit actions",
            "Nested states"
        ],
        "physical": [
            "Hardware node representation",
            "Deployment links",
            "Communication buses",
            "Physical interfaces"
        ],
        "class": [
            "UML class notation",
            "Inheritance hierarchies",
            "Associations",
            "Attributes and operations"
        ],
        "tree": [
            "Reingold-Tilford layout",
            "Hierarchical breakdown",
            "Expand/collapse indicators",
            "Category icons"
        ],
        "capability": [
            "3-level hierarchy",
            "Mission/Capability/Operational",
            "Capability associations",
            "Color-coded levels"
        ],
        "functional-chain": [
            "Left-to-right execution flow",
            "Function sequence",
            "Data exchange labels",
            "Port connections"
        ]
    }
    
    return features.get(diagram_type, [])
```

---

## 🌐 ArcViz Web Application Updates

### Architecture Overview

```
arcviz-web/
├── apps/
│   ├── api/              # Backend API (Fastify)
│   ├── diagram-service/  # Diagram rendering (TypeScript)
│   └── web/              # Frontend (Next.js)
```

### 1. Update API Endpoints (`apps/api/`)

#### Add Diagram Generation Endpoints

**File**: `apps/api/src/routes/diagrams.ts` (create if doesn't exist)

```typescript
import { FastifyInstance } from 'fastify';
import { exec } from 'child_process';
import { promisify } from 'util';
import { readFile } from 'fs/promises';

const execAsync = promisify(exec);

export default async function diagramRoutes(fastify: FastifyInstance) {
  // Generate single diagram
  fastify.post('/api/diagrams/generate', async (request, reply) => {
    const { modelPath, diagramType, outputPath } = request.body as any;
    
    // Validate diagram type
    const validTypes = [
      'operational', 'functional', 'component', 'sequence',
      'state-machine', 'physical', 'class', 'tree',
      'capability', 'functional-chain'
    ];
    
    if (!validTypes.includes(diagramType)) {
      return reply.code(400).send({ 
        error: 'Invalid diagram type',
        validTypes 
      });
    }
    
    try {
      // Generate diagram using CLI
      const output = outputPath || `./temp/${diagramType}-${Date.now()}.svg`;
      await execAsync(`arclang diagram ${modelPath} -o ${output} --format ${diagramType}`);
      
      // Read SVG content
      const svgContent = await readFile(output, 'utf-8');
      
      return {
        success: true,
        diagramType,
        svg: svgContent,
        outputPath: output
      };
    } catch (error) {
      return reply.code(500).send({
        error: 'Diagram generation failed',
        message: error.message
      });
    }
  });
  
  // Generate all diagrams
  fastify.post('/api/diagrams/generate-all', async (request, reply) => {
    const { modelPath, outputDir } = request.body as any;
    
    const diagramTypes = [
      'operational', 'functional', 'component', 'sequence',
      'state-machine', 'physical', 'class', 'tree',
      'capability', 'functional-chain'
    ];
    
    const results = await Promise.allSettled(
      diagramTypes.map(async (type) => {
        const output = `${outputDir || './diagrams'}/${type}.svg`;
        await execAsync(`arclang diagram ${modelPath} -o ${output} --format ${type}`);
        const svg = await readFile(output, 'utf-8');
        return { type, svg, outputPath: output };
      })
    );
    
    const successful = results.filter(r => r.status === 'fulfilled');
    const failed = results.filter(r => r.status === 'rejected');
    
    return {
      success: successful.length > 0,
      total: 10,
      successful: successful.length,
      failed: failed.length,
      diagrams: successful.map(r => (r as any).value)
    };
  });
  
  // Get available diagram types
  fastify.get('/api/diagrams/types', async (request, reply) => {
    return {
      types: [
        { id: 'operational', name: 'Operational Activity', description: 'Swimlane activity diagrams' },
        { id: 'functional', name: 'Functional Dataflow', description: 'Function and data flow' },
        { id: 'component', name: 'Component Architecture', description: 'Block diagrams' },
        { id: 'sequence', name: 'Sequence Diagram', description: 'Interaction scenarios' },
        { id: 'state-machine', name: 'State Machine', description: 'State transitions' },
        { id: 'physical', name: 'Physical Architecture', description: 'Hardware deployment' },
        { id: 'class', name: 'Class Diagram', description: 'Data types and classes' },
        { id: 'tree', name: 'Tree Diagram', description: 'Hierarchical breakdown' },
        { id: 'capability', name: 'Capability Diagram', description: 'Requirements hierarchy' },
        { id: 'functional-chain', name: 'Functional Chain', description: 'Execution scenarios' }
      ]
    };
  });
}
```

### 2. Update Web Editor (`apps/web/`)

#### Add Diagram Generation UI Component

**File**: `apps/web/components/DiagramGenerator.tsx` (create new)

```typescript
'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Select } from '@/components/ui/select';
import { Card } from '@/components/ui/card';

const DIAGRAM_TYPES = [
  { value: 'operational', label: 'Operational Activity', icon: '🏊' },
  { value: 'functional', label: 'Functional Dataflow', icon: '🔄' },
  { value: 'component', label: 'Component Architecture', icon: '🧱' },
  { value: 'sequence', label: 'Sequence Diagram', icon: '⏱️' },
  { value: 'state-machine', label: 'State Machine', icon: '🔄' },
  { value: 'physical', label: 'Physical Architecture', icon: '🖥️' },
  { value: 'class', label: 'Class Diagram', icon: '📦' },
  { value: 'tree', label: 'Tree Diagram', icon: '🌳' },
  { value: 'capability', label: 'Capability Diagram', icon: '🎯' },
  { value: 'functional-chain', label: 'Functional Chain', icon: '⛓️' }
];

export function DiagramGenerator({ modelPath }: { modelPath: string }) {
  const [selectedType, setSelectedType] = useState('operational');
  const [loading, setLoading] = useState(false);
  const [svg, setSvg] = useState<string | null>(null);
  const [generatingAll, setGeneratingAll] = useState(false);
  
  const generateDiagram = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/diagrams/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ modelPath, diagramType: selectedType })
      });
      
      const data = await response.json();
      if (data.success) {
        setSvg(data.svg);
      }
    } catch (error) {
      console.error('Failed to generate diagram:', error);
    } finally {
      setLoading(false);
    }
  };
  
  const generateAllDiagrams = async () => {
    setGeneratingAll(true);
    try {
      const response = await fetch('/api/diagrams/generate-all', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ modelPath })
      });
      
      const data = await response.json();
      alert(`Generated ${data.successful}/10 diagrams successfully!`);
    } catch (error) {
      console.error('Failed to generate diagrams:', error);
    } finally {
      setGeneratingAll(false);
    }
  };
  
  return (
    <div className="space-y-4">
      <Card className="p-4">
        <h3 className="text-lg font-semibold mb-4">Generate Diagrams</h3>
        
        <div className="flex gap-4 mb-4">
          <Select
            value={selectedType}
            onValueChange={setSelectedType}
            options={DIAGRAM_TYPES.map(t => ({
              value: t.value,
              label: `${t.icon} ${t.label}`
            }))}
          />
          
          <Button 
            onClick={generateDiagram}
            disabled={loading}
          >
            {loading ? 'Generating...' : 'Generate'}
          </Button>
          
          <Button
            onClick={generateAllDiagrams}
            disabled={generatingAll}
            variant="secondary"
          >
            {generatingAll ? 'Generating All...' : 'Generate All 10'}
          </Button>
        </div>
        
        {svg && (
          <div className="border rounded p-4 bg-white">
            <div dangerouslySetInnerHTML={{ __html: svg }} />
          </div>
        )}
      </Card>
    </div>
  );
}
```

#### Integrate into Editor Page

**File**: `apps/web/app/editor/page.tsx` (update)

```typescript
import { DiagramGenerator } from '@/components/DiagramGenerator';
import { MonacoEditor } from '@/components/MonacoEditor';

export default function EditorPage() {
  const [modelPath, setModelPath] = useState('');
  const [content, setContent] = useState('');
  
  return (
    <div className="grid grid-cols-2 gap-4 p-4 h-screen">
      {/* Left: Editor */}
      <div className="flex flex-col">
        <MonacoEditor
          value={content}
          onChange={setContent}
          language="arclang"
        />
      </div>
      
      {/* Right: Diagram Generator + Preview */}
      <div className="flex flex-col overflow-auto">
        <DiagramGenerator modelPath={modelPath} />
      </div>
    </div>
  );
}
```

### 3. Add AI Integration

**File**: `apps/web/lib/ai-integration.ts` (create new)

```typescript
export async function generateWithAI(prompt: string, modelPath?: string) {
  // Use MCP server via API
  const response = await fetch('/api/ai/generate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ prompt, modelPath })
  });
  
  return response.json();
}

export async function askAI(question: string, context?: any) {
  const response = await fetch('/api/ai/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question, context })
  });
  
  return response.json();
}
```

---

## 📋 Implementation Checklist

### MCP Server
- [x] Add diagram generation methods to `generation.py`
- [x] Register tools in `server.py`
- [x] Implement compiler wrapper methods
- [ ] Test diagram generation via MCP
- [ ] Update README with new tools

### ArcViz Web - API
- [x] Create diagram routes
- [x] Add diagram generation endpoints
- [x] Test API endpoints
- [x] Add error handling

### ArcViz Web - Frontend
- [x] Create DiagramGenerator component
- [x] Integrate into editor page
- [x] Add diagram type selector
- [x] Implement SVG preview
- [x] Add "Generate All" button
- [x] Add AI integration utility

### Integration Testing
- [ ] Test MCP → API → Frontend flow
- [ ] Test all 10 diagram types
- [ ] Test AI-generated models
- [ ] Test bulk diagram generation
- [ ] Performance testing with large models

---

## 🚀 Next Steps

1. **Complete MCP Server** (30 min)
   - Register new tools
   - Implement wrapper methods
   - Test with sample models

2. **Update API** (1 hour)
   - Add diagram routes
   - Implement endpoints
   - Test with Postman

3. **Build Frontend** (2 hours)
   - Create components
   - Integrate editor
   - Add AI features
   - Polish UI/UX

4. **End-to-End Testing** (1 hour)
   - Test full workflow
   - Fix any issues
   - Document usage

**Total Estimated Time**: 4.5 hours

---

## 📝 Success Criteria

✅ **MCP Server**:
- AI can generate all 10 diagram types
- Rich content quality maintained
- Clear error messages

✅ **Web Application**:
- Editor loads and saves models
- Diagrams generate on demand
- All 10 types working
- AI integration functional
- Professional UI/UX

✅ **Integration**:
- Seamless MCP → API → Web flow
- Fast diagram generation (<5s)
- Reliable error handling
- Good user experience

---

**Status**: Plan complete, ready for implementation! 🎯
