# Complete Capella Implementation for ArcLang
## Based on Actual Capella Diagram Analysis

This document provides the **complete implementation plan** for replicating all Capella modeling capabilities in ArcLang, based on analysis of real Capella diagrams.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│  ArcLang Parser (Rust)                                        │
│  ├─ Operational Layer Syntax                                 │
│  ├─ System Layer Syntax (Functions, Data Flows)              │
│  ├─ Logical Layer Syntax (Components, Allocation)            │
│  ├─ Physical Layer Syntax (Nodes, Links)                     │
│  ├─ Behavioral Syntax (States, Sequences)                    │
│  └─ Data Model Syntax (Exchange Items, Types)                │
└────────────────────┬─────────────────────────────────────────┘
                     │ JSON
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  Diagram Rendering Engine (TypeScript/React)                 │
│  ├─ Operational Activity Renderer (Swimlanes)                │
│  ├─ Capability Decomposition Renderer (Hierarchical)         │
│  ├─ Functional Dataflow Renderer (Functions + Ports)         │
│  ├─ Component Block Diagram Renderer (Nested Components)     │
│  ├─ Physical Node Renderer (Hardware + Software)             │
│  ├─ Sequence Diagram Renderer (Lifelines + Messages)         │
│  ├─ State Machine Renderer (States + Transitions)            │
│  └─ Data Model Renderer (Classes + Relationships)            │
└────────────────────┬─────────────────────────────────────────┘
                     │ SVG/Canvas
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  Interactive Viewer (Pan/Zoom/Export)                        │
└──────────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap (32 Weeks)

### **Phase 1: Foundation & Syntax** (Weeks 1-4)
### **Phase 2: Operational Layer** (Weeks 5-10)
### **Phase 3: System Layer** (Weeks 11-16)
### **Phase 4: Logical Layer** (Weeks 17-22)
### **Phase 5: Physical Layer** (Weeks 23-26)
### **Phase 6: Behavioral** (Weeks 27-30)
### **Phase 7: Polish** (Weeks 31-32)

---

## PHASE 1: Foundation & Syntax (Weeks 1-4)

### Week 1-2: Extend ArcLang Parser (Rust)

**New AST Node Types:**

```rust
// src/ast/operational.rs
pub struct OperationalAnalysis {
    pub name: String,
    pub entities: Vec<Entity>,
    pub activities: Vec<OperationalActivity>,
    pub exchanges: Vec<OperationalExchange>,
    pub capabilities: Vec<Capability>,
}

pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,  // Actor, System, Environment
    pub icon: String,  // "person", "airplane", "monitor"
}

pub enum EntityType {
    Actor,
    System,
    Environment,
}

pub struct OperationalActivity {
    pub id: String,
    pub name: String,
    pub performed_by: String,  // Entity ID
    pub category: String,
    pub icon: String,
    pub color: String,
    pub sub_activities: Vec<OperationalActivity>,
}

pub struct OperationalExchange {
    pub from: String,  // Activity ID
    pub to: String,    // Activity ID
    pub data_type: String,
    pub protocol: Option<String>,
}

pub struct Capability {
    pub id: String,
    pub name: String,
    pub level: CapabilityLevel,
    pub children: Vec<Capability>,
}

pub enum CapabilityLevel {
    Mission,
    Capability,
    SubCapability,
}

// src/ast/system.rs
pub struct SystemFunction {
    pub id: String,
    pub name: String,
    pub category: FunctionCategory,
    pub ports: Vec<FunctionPort>,
    pub sub_functions: Vec<SystemFunction>,
    pub color: String,
}

pub enum FunctionCategory {
    Environmental,   // Blue
    System,          // Green
    Management,      // Green
    Control,         // Green
    Interaction,     // Green
}

pub struct FunctionPort {
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortType,
    pub data_type: String,
}

pub enum PortDirection {
    In,
    Out,
    InOut,
}

pub enum PortType {
    Data,
    Control,
    Event,
}

pub struct FunctionalExchange {
    pub from_port: String,  // "FunctionID.PortName"
    pub to_port: String,
    pub data_type: String,
}

// src/ast/logical.rs
pub struct LogicalComponent {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub sub_components: Vec<LogicalComponent>,
    pub allocated_functions: Vec<String>,  // Function IDs
    pub ports: Vec<ComponentPort>,
    pub color: String,
}

pub struct ComponentPort {
    pub name: String,
    pub direction: PortDirection,
    pub interface_type: String,
}

pub struct ComponentExchange {
    pub from_port: String,
    pub to_port: String,
    pub exchange_item: String,
}

// src/ast/physical.rs
pub struct PhysicalNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub behavior_components: Vec<BehaviorComponent>,
    pub hardware_specs: Option<HardwareSpecs>,
}

pub enum NodeType {
    Hardware,
    Software,
    SystemOfSystems,
}

pub struct BehaviorComponent {
    pub id: String,
    pub name: String,
    pub allocated_functions: Vec<String>,
    pub color: String,
}

pub struct PhysicalLink {
    pub from: String,
    pub to: String,
    pub protocol: String,
    pub bandwidth: Option<String>,
}

pub struct PhysicalExchange {
    pub from: String,
    pub to: String,
    pub via: String,  // Physical link ID
    pub message_type: String,
    pub frequency: Option<String>,
}

// src/ast/behavioral.rs
pub struct StateMachine {
    pub name: String,
    pub initial_state: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
}

pub struct State {
    pub name: String,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
    pub internal_transitions: Vec<InternalTransition>,
    pub sub_states: Option<Vec<State>>,
    pub color: String,
}

pub struct Transition {
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub timing: Option<String>,
}

pub struct Scenario {
    pub name: String,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub fragments: Vec<CombinedFragment>,
}

pub struct Participant {
    pub id: String,
    pub name: String,
    pub participant_type: ParticipantType,
    pub lifeline_color: String,
}

pub enum ParticipantType {
    Actor,
    Component,
    System,
}

pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
    pub message_type: MessageType,
    pub activation: bool,
    pub timing: Option<String>,
}

pub enum MessageType {
    Synchronous,
    Asynchronous,
    Return,
}

pub struct CombinedFragment {
    pub fragment_type: FragmentType,
    pub label: String,
    pub condition: Option<String>,
    pub operands: Vec<FragmentOperand>,
}

pub enum FragmentType {
    Par,   // Parallel
    Opt,   // Optional
    Loop,  // Loop
    Alt,   // Alternative
}

pub struct FragmentOperand {
    pub label: String,
    pub messages: Vec<Message>,
}

// src/ast/datamodel.rs
pub struct ExchangeItem {
    pub name: String,
    pub stereotype: String,  // «datatype», «interface», etc.
    pub attributes: Vec<Attribute>,
}

pub struct Attribute {
    pub name: String,
    pub attr_type: String,
    pub default_value: Option<String>,
    pub enumeration: Option<Vec<String>>,
}

pub struct DataType {
    pub name: String,
    pub base_type: Option<String>,
    pub enumeration_values: Option<Vec<EnumValue>>,
}

pub struct EnumValue {
    pub name: String,
    pub value: Option<String>,
}
```

### Week 3: JSON Export Schema

```typescript
// apps/web/lib/types/capella-model.ts
export interface CapellaModel {
    operational: OperationalAnalysis
    system: SystemAnalysis
    logical: LogicalArchitecture
    physical: PhysicalArchitecture
    behavioral: BehavioralModels
    dataModel: DataModel
    traceability: TraceabilityLinks
}

export interface OperationalAnalysis {
    entities: Entity[]
    activities: OperationalActivity[]
    exchanges: OperationalExchange[]
    capabilities: Capability[]
}

// ... (full type definitions matching Rust AST)
```

### Week 4: Setup Diagram Service

```bash
cd /Users/malek/Arclang/arcviz-web
mkdir -p apps/diagram-service/src/{renderers,layouts,svg,utils}

# package.json
{
  "name": "@arcviz/diagram-service",
  "dependencies": {
    "elkjs": "^0.8.2",
    "d3": "^7.8.5",
    "dagre": "^0.8.5",
    "@types/node": "^20.0.0"
  }
}
```

---

## PHASE 2: Operational Layer (Weeks 5-10)

### Week 5-6: Operational Activity Diagram with Swimlanes

**Complete ArcLang Syntax:**

```arc
operational_analysis "In-Flight Entertainment System" {
    // Define operational entities (create swimlanes)
    entity Passenger {
        id: "OE-001"
        type: actor
        icon: "person"
        description: "Aircraft passenger using IFE"
    }
    
    entity IFESystem {
        id: "OE-002"
        type: system
        icon: "monitor"
        description: "In-Flight Entertainment System"
    }
    
    entity Aircraft {
        id: "OE-003"
        type: system
        icon: "airplane"
        description: "Aircraft systems"
    }
    
    entity CabinCrew {
        id: "OE-004"
        type: actor
        icon: "person"
        description: "Flight attendants"
    }
    
    // Operational activities (with icons like Capella)
    operational_activity "Listen to Audio" {
        id: "OA-001"
        performed_by: Passenger
        category: "entertainment"
        icon: "headphones"
        color: "#FFD966"  // Yellow boxes
        
        // Sub-activities (hierarchical decomposition)
        sub_activity "Select Audio Channel" {
            id: "OA-001-1"
            icon: "list"
        }
        
        sub_activity "Adjust Volume" {
            id: "OA-001-2"
            icon: "volume"
        }
    }
    
    operational_activity "Watch Movie" {
        id: "OA-002"
        performed_by: Passenger
        category: "entertainment"
        icon: "film"
        color: "#FFD966"
    }
    
    operational_activity "Play Games" {
        id: "OA-003"
        performed_by: Passenger
        category: "entertainment"
        icon: "gamepad"
        color: "#FFD966"
    }
    
    operational_activity "Provide Audio Content" {
        id: "OA-004"
        performed_by: IFESystem
        category: "service"
        icon: "server"
        color: "#A9D18E"  // Green for system
    }
    
    operational_activity "Provide Connectivity" {
        id: "OA-005"
        performed_by: IFESystem
        category: "service"
        icon: "wifi"
        color: "#A9D18E"
    }
    
    operational_activity "Broadcast Audio" {
        id: "OA-006"
        performed_by: Aircraft
        category: "communication"
        icon: "broadcast"
        color: "#B4C7E7"  // Blue for aircraft
    }
    
    operational_activity "Provide Moving Map" {
        id: "OA-007"
        performed_by: IFESystem
        category: "information"
        icon: "map"
        color: "#A9D18E"
    }
    
    operational_activity "Manage Content" {
        id: "OA-008"
        performed_by: CabinCrew
        category: "management"
        icon: "settings"
        color: "#FFE699"  // Light yellow for crew
    }
    
    // Data flows (labeled arrows)
    operational_exchange "Audio Selection" {
        from: "OA-001"
        to: "OA-004"
        data_type: "Channel_Selection"
        label: "Audio Selection"
    }
    
    operational_exchange "Audio Stream" {
        from: "OA-004"
        to: "OA-001"
        data_type: "Audio_Data"
        label: "Audio Stream"
    }
    
    operational_exchange "Movie Selection" {
        from: "OA-002"
        to: "OA-004"
        data_type: "Movie_Request"
        label: "Movie Selection"
    }
    
    operational_exchange "Chosen Movie" {
        from: "OA-004"
        to: "OA-002"
        data_type: "Video_Stream"
        label: "Chosen Movie"
    }
    
    operational_exchange "Audio Announcement" {
        from: "OA-006"
        to: "OA-004"
        data_type: "PA_Message"
        label: "Audio Announcement"
    }
    
    operational_exchange "Moving Map" {
        from: "OA-007"
        to: "OA-002"
        data_type: "Map_Data"
        label: "Moving Map"
    }
    
    operational_exchange "Content Update" {
        from: "OA-008"
        to: "OA-004"
        data_type: "Content_Catalog"
        label: "Content Update"
    }
}
```

**Renderer Implementation:**

```typescript
// apps/diagram-service/src/renderers/operational-activity-renderer.ts
import { OperationalAnalysis } from '../types'
import { SwimlaneLayout } from '../layouts/swimlane-layout'
import { createSVG, createRect, createText, createPath, createGroup } from '../svg/primitives'

export class OperationalActivityRenderer {
    private layout: SwimlaneLayout
    private LANE_HEIGHT = 220
    private LANE_HEADER_WIDTH = 180
    private ACTIVITY_WIDTH = 160
    private ACTIVITY_HEIGHT = 90
    
    constructor(private model: OperationalAnalysis) {
        this.layout = new SwimlaneLayout()
    }
    
    render(): string {
        const layoutResult = this.layout.layout(this.model)
        const svg = createSVG(layoutResult.width, layoutResult.height)
        
        // 1. Draw swimlanes (horizontal bands)
        this.renderSwimlanes(svg, layoutResult)
        
        // 2. Draw activities (yellow boxes with icons)
        this.renderActivities(svg, layoutResult)
        
        // 3. Draw data flows (labeled arrows)
        this.renderDataFlows(svg, layoutResult)
        
        return svg.toString()
    }
    
    private renderSwimlanes(svg: SVGElement, layout: LayoutResult) {
        let yOffset = 0
        
        for (const entity of this.model.entities) {
            // Lane background (alternating colors for clarity)
            const laneColor = entity.type === 'actor' ? '#E8F4F8' : '#F5F5F5'
            const laneBg = createRect({
                x: 0,
                y: yOffset,
                width: layout.width,
                height: this.LANE_HEIGHT,
                fill: laneColor,
                stroke: '#CCCCCC',
                strokeWidth: 1.5
            })
            svg.appendChild(laneBg)
            
            // Lane header (darker blue/gray)
            const headerBg = createRect({
                x: 0,
                y: yOffset,
                width: this.LANE_HEADER_WIDTH,
                height: this.LANE_HEIGHT,
                fill: entity.type === 'actor' ? '#4472C4' : '#7F7F7F',
                stroke: '#333333',
                strokeWidth: 2
            })
            svg.appendChild(headerBg)
            
            // Entity icon (stick figure for actors)
            if (entity.icon === 'person') {
                const icon = this.createStickFigure(
                    this.LANE_HEADER_WIDTH / 2,
                    yOffset + 80
                )
                svg.appendChild(icon)
            } else {
                // System/airplane icons (simple geometric shapes)
                const icon = this.createSystemIcon(
                    entity.icon,
                    this.LANE_HEADER_WIDTH / 2,
                    yOffset + 70
                )
                svg.appendChild(icon)
            }
            
            // Entity name (white text, centered)
            const nameText = createText({
                x: this.LANE_HEADER_WIDTH / 2,
                y: yOffset + this.LANE_HEIGHT - 20,
                text: entity.name,
                fontSize: 14,
                fontWeight: 'bold',
                fill: 'white',
                textAnchor: 'middle'
            })
            svg.appendChild(nameText)
            
            yOffset += this.LANE_HEIGHT
        }
    }
    
    private renderActivities(svg: SVGElement, layout: LayoutResult) {
        for (const activity of layout.positionedActivities) {
            const group = createGroup()
            
            // Activity box (rounded yellow rectangle - Capella style)
            const rect = createRect({
                x: activity.x,
                y: activity.y,
                width: this.ACTIVITY_WIDTH,
                height: this.ACTIVITY_HEIGHT,
                rx: 12,  // Rounded corners
                ry: 12,
                fill: activity.color,
                stroke: '#BF8F00',
                strokeWidth: 2.5
            })
            group.appendChild(rect)
            
            // Circular icon background (white circle, top-left)
            const iconCircle = createCircle({
                cx: activity.x + 30,
                cy: activity.y + 28,
                r: 18,
                fill: 'white',
                stroke: '#BF8F00',
                strokeWidth: 2
            })
            group.appendChild(iconCircle)
            
            // Activity icon (custom SVG path for each icon type)
            const icon = this.getActivityIcon(activity.icon)
            icon.setAttribute('transform', `translate(${activity.x + 18}, ${activity.y + 16})`)
            group.appendChild(icon)
            
            // Activity name (wrapped text, centered)
            const name = this.wrapText(activity.name, this.ACTIVITY_WIDTH - 20)
            const nameText = createText({
                x: activity.x + this.ACTIVITY_WIDTH / 2,
                y: activity.y + 55,
                text: name,
                fontSize: 12,
                textAnchor: 'middle',
                fontFamily: 'Arial, sans-serif'
            })
            group.appendChild(nameText)
            
            // Expansion indicator (if has sub-activities)
            if (activity.subActivities && activity.subActivities.length > 0) {
                const expandIcon = createPath({
                    d: `M ${activity.x + this.ACTIVITY_WIDTH - 25} ${activity.y + this.ACTIVITY_HEIGHT - 20} 
                        l 8 0 l -4 6 z`,
                    fill: '#666',
                    stroke: 'none'
                })
                group.appendChild(expandIcon)
            }
            
            svg.appendChild(group)
        }
    }
    
    private renderDataFlows(svg: SVGElement, layout: LayoutResult) {
        for (const exchange of this.model.exchanges) {
            const source = layout.activityPositions.get(exchange.from)
            const target = layout.activityPositions.get(exchange.to)
            
            if (!source || !target) continue
            
            // Calculate arrow path (curved for better readability)
            const x1 = source.x + this.ACTIVITY_WIDTH
            const y1 = source.y + this.ACTIVITY_HEIGHT / 2
            const x2 = target.x
            const y2 = target.y + this.ACTIVITY_HEIGHT / 2
            
            // Curved arrow (Bezier curve)
            const midX = (x1 + x2) / 2
            const curve = y1 === y2 ? 0 : 30  // Curve amount
            
            const arrowPath = createPath({
                d: `M ${x1} ${y1} 
                    Q ${midX} ${y1 + curve} ${midX} ${(y1 + y2) / 2}
                    Q ${midX} ${y2 - curve} ${x2} ${y2}`,
                fill: 'none',
                stroke: '#4472C4',
                strokeWidth: 2.5,
                markerEnd: 'url(#arrowhead)'
            })
            svg.appendChild(arrowPath)
            
            // Flow label (data type name)
            const labelBg = createRect({
                x: midX - 50,
                y: (y1 + y2) / 2 - 12,
                width: 100,
                height: 20,
                rx: 4,
                fill: 'white',
                stroke: '#4472C4',
                strokeWidth: 1
            })
            svg.appendChild(labelBg)
            
            const label = createText({
                x: midX,
                y: (y1 + y2) / 2 + 3,
                text: exchange.label || exchange.dataType,
                fontSize: 10,
                fill: '#2E5C8A',
                textAnchor: 'middle',
                fontStyle: 'italic'
            })
            svg.appendChild(label)
        }
        
        // Define arrowhead marker
        const defs = svg.querySelector('defs') || svg.appendChild(document.createElementNS(SVG_NS, 'defs'))
        const marker = `
            <marker id="arrowhead" markerWidth="10" markerHeight="10" 
                    refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                <path d="M0,0 L0,6 L9,3 z" fill="#4472C4" />
            </marker>
        `
        defs.innerHTML += marker
    }
    
    private createStickFigure(x: number, y: number): SVGGElement {
        const group = createGroup()
        
        // Head (circle)
        const head = createCircle({
            cx: x,
            cy: y - 25,
            r: 16,
            fill: 'white',
            stroke: 'white',
            strokeWidth: 3
        })
        group.appendChild(head)
        
        // Body (vertical line)
        const body = createPath({
            d: `M ${x} ${y - 9} L ${x} ${y + 22}`,
            stroke: 'white',
            strokeWidth: 4,
            strokeLinecap: 'round'
        })
        group.appendChild(body)
        
        // Arms (horizontal line)
        const arms = createPath({
            d: `M ${x - 18} ${y + 2} L ${x + 18} ${y + 2}`,
            stroke: 'white',
            strokeWidth: 4,
            strokeLinecap: 'round'
        })
        group.appendChild(arms)
        
        // Legs (diagonal lines)
        const leftLeg = createPath({
            d: `M ${x} ${y + 22} L ${x - 12} ${y + 42}`,
            stroke: 'white',
            strokeWidth: 4,
            strokeLinecap: 'round'
        })
        const rightLeg = createPath({
            d: `M ${x} ${y + 22} L ${x + 12} ${y + 42}`,
            stroke: 'white',
            strokeWidth: 4,
            strokeLinecap: 'round'
        })
        group.appendChild(leftLeg)
        group.appendChild(rightLeg)
        
        return group
    }
    
    private createSystemIcon(iconType: string, x: number, y: number): SVGGElement {
        const group = createGroup()
        
        switch (iconType) {
            case 'monitor':
                // Computer monitor icon
                const screen = createRect({
                    x: x - 25,
                    y: y - 20,
                    width: 50,
                    height: 35,
                    rx: 3,
                    fill: 'white',
                    stroke: 'white',
                    strokeWidth: 3
                })
                const stand = createRect({
                    x: x - 8,
                    y: y + 15,
                    width: 16,
                    height: 10,
                    fill: 'white'
                })
                group.appendChild(screen)
                group.appendChild(stand)
                break
                
            case 'airplane':
                // Simple airplane icon
                const plane = createPath({
                    d: `M ${x} ${y - 10} 
                        L ${x + 25} ${y + 10} L ${x + 15} ${y + 12}
                        L ${x} ${y + 5} 
                        L ${x - 15} ${y + 12} L ${x - 25} ${y + 10} Z`,
                    fill: 'white',
                    stroke: 'white',
                    strokeWidth: 2
                })
                group.appendChild(plane)
                break
                
            default:
                // Generic system box
                const box = createRect({
                    x: x - 20,
                    y: y - 20,
                    width: 40,
                    height: 40,
                    rx: 5,
                    fill: 'white',
                    stroke: 'white',
                    strokeWidth: 3
                })
                group.appendChild(box)
        }
        
        return group
    }
    
    private getActivityIcon(iconType: string): SVGElement {
        // SVG paths for activity icons (headphones, film, gamepad, etc.)
        const icons: Record<string, string> = {
            'headphones': 'M12,3a9,9,0,0,0-9,9v7a3,3,0,0,0,6,0V16a3,3,0,0,0-6,0V12A9,9,0,0,1,21,12v7a3,3,0,0,0-6,0v3a3,3,0,0,0,6,0V12A9,9,0,0,0,12,3Z',
            'film': 'M4,4H6V6H4V4M18,4h2V6H18V4M4,8H6v2H4V8M18,8h2v2H18V8M4,12H6v2H4V12M18,12h2v2H18V12M4,16H6v2H4V16M18,16h2v2H18V16M4,20H6v2H4V20M18,20h2v2H18V20M8,2H16a2,2,0,0,1,2,2V20a2,2,0,0,1-2,2H8a2,2,0,0,1-2-2V4A2,2,0,0,1,8,2Z',
            'gamepad': 'M6,9H8v2h2v2H8v2H6V13H4V11H6M14,9h2v4H14M18,11h2v2H18M16,4A7.74,7.74,0,0,0,8,7.68,7.74,7.74,0,0,0,0,15a7.72,7.72,0,0,0,3,6.08A7.88,7.88,0,0,0,8,23a7.75,7.75,0,0,0,8-8,7.74,7.74,0,0,0-8-7.68A7.88,7.88,0,0,0,3,21.08Z',
            'server': 'M4,1H20A1,1,0,0,1,21,2V6A1,1,0,0,1,20,7H4A1,1,0,0,1,3,6V2A1,1,0,0,1,4,1M4,9H20A1,1,0,0,1,21,10V14A1,1,0,0,1,20,15H4A1,1,0,0,1,3,14V10A1,1,0,0,1,4,9M4,17H20A1,1,0,0,1,21,18V22A1,1,0,0,1,20,23H4A1,1,0,0,1,3,22V18A1,1,0,0,1,4,17M6,4V6H8V4H6M6,12V14H8V12H6M6,20V22H8V20H6Z',
            'wifi': 'M12,21L15.6,16.2C14.6,15.45 13.35,15 12,15C10.65,15 9.4,15.45 8.4,16.2L12,21M12,3C7.95,3 4.21,4.34 1.2,6.6L3,9C5.5,7.12 8.62,6 12,6C15.38,6 18.5,7.12 21,9L22.8,6.6C19.79,4.34 16.05,3 12,3M12,9C9.3,9 6.81,9.89 4.8,11.4L6.6,13.8C8.1,12.67 9.97,12 12,12C14.03,12 15.9,12.67 17.4,13.8L19.2,11.4C17.19,9.89 14.7,9 12,9Z',
            'map': 'M15,19L9,16.89V5L15,7.11M20.5,3C20.44,3 20.39,3 20.34,3L15,5.1L9,3L3.36,4.9C3.15,4.97 3,5.15 3,5.38V20.5A0.5,0.5 0 0,0 3.5,21C3.55,21 3.61,21 3.66,20.97L9,18.9L15,21L20.64,19.1C20.85,19.03 21,18.85 21,18.62V3.5A0.5,0.5 0 0,0 20.5,3Z',
            'settings': 'M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.21,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.21,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.67 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z'
        }
        
        const path = document.createElementNS(SVG_NS, 'path')
        path.setAttribute('d', icons[iconType] || icons['settings'])
        path.setAttribute('fill', '#666')
        path.setAttribute('transform', 'scale(0.6)')
        
        return path
    }
    
    private wrapText(text: string, maxWidth: number): string[] {
        const words = text.split(' ')
        const lines: string[] = []
        let currentLine = ''
        
        for (const word of words) {
            const testLine = currentLine + (currentLine ? ' ' : '') + word
            if (this.measureText(testLine) > maxWidth) {
                if (currentLine) lines.push(currentLine)
                currentLine = word
            } else {
                currentLine = testLine
            }
        }
        if (currentLine) lines.push(currentLine)
        
        return lines
    }
    
    private measureText(text: string, fontSize: number = 12): number {
        // Approximate text width (7 pixels per character for Arial 12px)
        return text.length * 7
    }
}
```

### Week 7-8: Capability Decomposition

```arc
operational_analysis "IFE Capabilities" {
    // Top-level mission
    capability "Provide Cabin Management Solutions" {
        id: "CAP-001"
        level: mission
        color: "#F4B084"
        stereotype: "module"
        
        // Child capabilities
        capability "Provide Audio and Video Intercommunication Means" {
            id: "CAP-001-1"
            level: capability
            stereotype: "capability"
        }
        
        capability "Provide Moving-Map Services" {
            id: "CAP-001-2"
            level: capability
            stereotype: "capability"
        }
    }
    
    capability "Provide Entertainment Solutions" {
        id: "CAP-002"
        level: mission
        color: "#F4B084"
        stereotype: "module"
        
        capability "Provide Audio Entertainment Services" {
            id: "CAP-002-1"
            level: capability
        }
        
        capability "Provide Video Entertainment Services" {
            id: "CAP-002-2"
            level: capability
            
            // Sub-capabilities
            capability "Stream Movies" {
                id: "CAP-002-2-1"
                level: sub_capability
            }
            
            capability "Display Moving Map" {
                id: "CAP-002-2-2"
                level: sub_capability
            }
        }
        
        capability "Provide Video Gaming Services" {
            id: "CAP-002-3"
            level: capability
        }
    }
    
    // Capability relationships
    capability_association {
        from: "CAP-002-2"
        to: "CAP-001-2"
        type: "uses"
        label: "uses moving map data"
    }
}
```

### Week 9-10: Actor Relationships & Capability Providers

```arc
operational_analysis "IFE Ecosystem" {
    // Actors
    actor Aircraft {
        id: "ACT-001"
        icon: "airplane"
    }
    
    actor CabinCrew {
        id: "ACT-002"
        icon: "person"
    }
    
    actor Passenger {
        id: "ACT-003"
        icon: "person"
    }
    
    // Systems (capability providers)
    system IFEServer {
        id: "SYS-001"
        stereotype: "module"
        provides: ["CAP-001", "CAP-002"]
    }
    
    // Actor-capability relationships
    actor_interaction {
        actor: "ACT-003"
        uses: "CAP-002-1"  // Passenger uses Audio Entertainment
    }
    
    actor_interaction {
        actor: "ACT-002"
        manages: "CAP-001"  // Crew manages Cabin Management
    }
}
```

---

## PHASE 3: System Layer (Weeks 11-16)

### Week 11-12: Functional Dataflow Diagrams

**Complete Camera Example:**

```arc
system_analysis "Digital Camera System" {
    // External actors (blue boxes)
    external_actor User {
        id: "EXT-001"
        color: "#5B9BD5"
    }
    
    external_actor Camera {
        id: "EXT-002"
        color: "#5B9BD5"
    }
    
    external_actor Environment {
        id: "EXT-003"
        color: "#5B9BD5"
    }
    
    // System functions (green rounded rectangles with icons)
    system_function "Provide environment and light" {
        id: "SF-001"
        category: environmental  // Blue
        color: "#4472C4"
        icon: "sun"
        
        // Functional ports (small squares on border)
        port light {
            direction: OUT
            type: data
            data_type: "Light_Photons"
        }
        
        // Sub-functions
        sub_function "Generate light" {
            id: "SF-001-1"
            port light_out {
                direction: OUT
                type: data
            }
        }
        
        sub_function "Reflect light" {
            id: "SF-001-2"
            port light_in {
                direction: IN
                type: data
            }
            port reflected_light {
                direction: OUT
                type: data
            }
        }
    }
    
    system_function "Acquire image" {
        id: "SF-002"
        category: system  // Green
        color: "#70AD47"
        icon: "camera"
        
        port reflected_light {
            direction: IN
            type: data
        }
        
        port raw_image {
            direction: OUT
            type: data
            data_type: "Image_Data"
        }
        
        sub_function "Focus image" {
            id: "SF-002-1"
        }
        
        sub_function "Detect image" {
            id: "SF-002-2"
        }
        
        sub_function "Generate Flash" {
            id: "SF-002-3"
            
            port flash_cmd {
                direction: IN
                type: control
            }
            
            port flash_light {
                direction: OUT
                type: data
            }
        }
    }
    
    system_function "Operate the camera" {
        id: "SF-003"
        category: interaction  // Green
        color: "#A9D08E"
        icon: "hand"
        
        port manual_action {
            direction: IN
            type: control
            data_type: "User_Command"
        }
        
        port shooting_cmd {
            direction: OUT
            type: control
        }
        
        port flash_cmd {
            direction: OUT
            type: control
        }
        
        sub_function "Select ON-OFF" {
            id: "SF-003-1"
        }
        
        sub_function "View Image" {
            id: "SF-003-2"
            port image {
                direction: IN
                type: data
            }
        }
        
        sub_function "Take picture" {
            id: "SF-003-3"
        }
        
        sub_function "Set shooting parameters" {
            id: "SF-003-4"
        }
    }
    
    system_function "Control the camera" {
        id: "SF-004"
        category: control  // Green
        color: "#70AD47"
        icon: "cpu"
        
        port shooting_cmd {
            direction: IN
            type: control
        }
        
        port raw_image {
            direction: IN
            type: data
        }
        
        port processed_image {
            direction: OUT
            type: data
        }
        
        sub_function "Process image" {
            id: "SF-004-1"
        }
        
        sub_function "Control flash" {
            id: "SF-004-2"
        }
        
        sub_function "Control shooting" {
            id: "SF-004-3"
        }
        
        sub_function "Store image" {
            id: "SF-004-4"
        }
    }
    
    system_function "Manage energy" {
        id: "SF-005"
        category: management  // Green
        color: "#92D050"
        icon: "battery"
        
        sub_function "Generate power" {
            id: "SF-005-1"
        }
        
        sub_function "Switch power" {
            id: "SF-005-2"
        }
    }
    
    system_function "Handle user interactions" {
        id: "SF-006"
        category: interaction
        color: "#A9D08E"
        
        sub_function "Display image" {
            id: "SF-006-1"
        }
        
        sub_function "Capture shutter activation" {
            id: "SF-006-2"
        }
        
        sub_function "Capture parameters" {
            id: "SF-006-3"
        }
    }
    
    // Functional exchanges (port-to-port connections with labels)
    functional_exchange "Light Generation" {
        from: "SF-001-1.light_out"
        to: "SF-001-2.light_in"
        data_type: "Light_Photons"
    }
    
    functional_exchange "Reflected Light" {
        from: "SF-001.light"
        to: "SF-002.reflected_light"
        data_type: "Light_Photons"
    }
    
    functional_exchange "Manual Actions" {
        from: "EXT-001"  // User
        to: "SF-003.manual_action"
        data_type: "User_Command"
    }
    
    functional_exchange "Shooting Command" {
        from: "SF-003.shooting_cmd"
        to: "SF-004.shooting_cmd"
        data_type: "Shooting_Command"
    }
    
    functional_exchange "Flash Command" {
        from: "SF-003.flash_cmd"
        to: "SF-002-3.flash_cmd"
        data_type: "Flash_Command"
    }
    
    functional_exchange "Raw Image" {
        from: "SF-002.raw_image"
        to: "SF-004.raw_image"
        data_type: "Image_Raw"
    }
    
    functional_exchange "Processed Image" {
        from: "SF-004.processed_image"
        to: "SF-003-2.image"
        data_type: "Image_Processed"
    }
}
```

**Renderer with Ports:**

```typescript
// apps/diagram-service/src/renderers/functional-dataflow-renderer.ts
export class FunctionalDataflowRenderer {
    private PORT_SIZE = 10
    private FUNCTION_WIDTH = 180
    private FUNCTION_HEIGHT = 100
    
    private renderFunction(func: SystemFunction, x: number, y: number): SVGGElement {
        const group = createGroup()
        
        // Function box (green rounded rectangle)
        const rect = createRect({
            x,
            y,
            width: this.FUNCTION_WIDTH,
            height: this.FUNCTION_HEIGHT,
            rx: 15,
            fill: func.color,
            stroke: '#548235',
            strokeWidth: 2.5
        })
        group.appendChild(rect)
        
        // Circular icon (top-left, white circle)
        const iconCircle = createCircle({
            cx: x + 28,
            cy: y + 25,
            r: 16,
            fill: 'white',
            stroke: '#548235',
            strokeWidth: 2
        })
        group.appendChild(iconCircle)
        
        // Function icon
        const icon = this.getFunctionIcon(func.icon)
        icon.setAttribute('transform', `translate(${x + 18}, ${y + 15})`)
        group.appendChild(icon)
        
        // Function name
        const name = createText({
            x: x + this.FUNCTION_WIDTH / 2,
            y: y + 60,
            text: func.name,
            fontSize: 12,
            fontWeight: 'bold',
            textAnchor: 'middle'
        })
        group.appendChild(name)
        
        // Render ports (small squares on borders)
        for (const port of func.ports) {
            const portElement = this.renderPort(port, func, x, y)
            group.appendChild(portElement)
        }
        
        return group
    }
    
    private renderPort(port: FunctionPort, func: SystemFunction, fx: number, fy: number): SVGGElement {
        const group = createGroup()
        
        // Calculate port position based on direction
        let px, py
        switch (port.direction) {
            case 'IN':
                px = fx  // Left side
                py = fy + this.FUNCTION_HEIGHT / 2
                break
            case 'OUT':
                px = fx + this.FUNCTION_WIDTH  // Right side
                py = fy + this.FUNCTION_HEIGHT / 2
                break
            case 'INOUT':
                px = fx + this.FUNCTION_WIDTH / 2  // Top
                py = fy
                break
        }
        
        // Port square
        const portRect = createRect({
            x: px - this.PORT_SIZE / 2,
            y: py - this.PORT_SIZE / 2,
            width: this.PORT_SIZE,
            height: this.PORT_SIZE,
            fill: port.type === 'control' ? '#C00000' : '#4472C4',
            stroke: '#000',
            strokeWidth: 1.5
        })
        group.appendChild(portRect)
        
        // Port name (small label)
        const label = createText({
            x: port.direction === 'IN' ? px - 15 : px + 15,
            y: py + 4,
            text: port.name,
            fontSize: 9,
            textAnchor: port.direction === 'IN' ? 'end' : 'start'
        })
        group.appendChild(label)
        
        return group
    }
    
    private renderFunctionalExchange(exchange: FunctionalExchange): SVGGElement {
        const group = createGroup()
        
        // Parse "FunctionID.PortName" format
        const [sourceFunc, sourcePort] = exchange.fromPort.split('.')
        const [targetFunc, targetPort] = exchange.toPort.split('.')
        
        const sourcePos = this.getPortPosition(sourceFunc, sourcePort)
        const targetPos = this.getPortPosition(targetFunc, targetPort)
        
        // Connection line (orthogonal routing like Capella)
        const path = this.createOrthogonalPath(sourcePos, targetPos)
        const line = createPath({
            d: path,
            fill: 'none',
            stroke: '#4472C4',
            strokeWidth: 2,
            markerEnd: 'url(#arrowhead)'
        })
        group.appendChild(line)
        
        // Data type label on arrow
        const midPoint = this.getMidPoint(sourcePos, targetPos)
        const labelBg = createRect({
            x: midPoint.x - 40,
            y: midPoint.y - 10,
            width: 80,
            height: 18,
            rx: 3,
            fill: 'white',
            stroke: '#4472C4',
            strokeWidth: 1
        })
        group.appendChild(labelBg)
        
        const label = createText({
            x: midPoint.x,
            y: midPoint.y + 3,
            text: exchange.dataType,
            fontSize: 9,
            textAnchor: 'middle',
            fontStyle: 'italic'
        })
        group.appendChild(label)
        
        return group
    }
    
    private createOrthogonalPath(from: Point, to: Point): string {
        // Manhattan routing (90-degree angles only)
        const midX = (from.x + to.x) / 2
        
        return `M ${from.x} ${from.y} 
                L ${midX} ${from.y}
                L ${midX} ${to.y}
                L ${to.x} ${to.y}`
    }
}
```

### Week 13-14: Function Hierarchy Tree View

```typescript
// Separate tree view renderer for functional breakdown
export class FunctionalHierarchyRenderer {
    render(functions: SystemFunction[]): string {
        const svg = createSVG(800, 1200)
        
        // Use Reingold-Tilford algorithm for tree layout
        const tree = this.buildTree(functions)
        const positioned = this.layoutTree(tree)
        
        // Render nodes and connections
        this.renderTreeNodes(svg, positioned)
        this.renderTreeEdges(svg, positioned)
        
        return svg.toString()
    }
    
    private buildTree(functions: SystemFunction[]): TreeNode {
        // Build parent-child hierarchy
        // ...
    }
    
    private layoutTree(root: TreeNode): PositionedTree {
        // Apply Reingold-Tilford layout algorithm
        // Ensures:
        // - No overlapping nodes
        // - Parents centered above children
        // - Aesthetic spacing
        // ...
    }
}
```

### Week 15-16: External Actors & Data Type Rendering

---

## PHASE 4: Logical Layer (Weeks 17-22)

### Week 17-18: Component Block Diagrams

**Complete Camera Logical Architecture:**

```arc
logical_architecture "Camera Logical Components" {
    // Top-level component (Camera System)
    component Camera {
        id: "LC-001"
        type: "System"
        color: "#5B9BD5"
        
        // Sub-components (nested inside)
        component "Camera power button" {
            id: "LC-001-1"
            type: "Hardware"
            color: "#9DC3E6"
            
            // Allocated function (green box inside)
            allocates: "SF-003-1"  // Select ON-OFF
        }
        
        component Battery {
            id: "LC-001-2"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: ["SF-005-1", "SF-005-2"]  // Generate/Switch power
        }
        
        component "Image Processor" {
            id: "LC-001-3"
            type: "Software"
            color: "#5B9BD5"
            
            allocates: "SF-004-1"  // Process image
        }
        
        component "Shutter-Sensor" {
            id: "LC-001-4"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: "SF-002-2"  // Detect image
        }
        
        component "Optical Assembly" {
            id: "LC-001-5"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: "SF-002-1"  // Focus image
        }
        
        component "LCD screen" {
            id: "LC-001-6"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: "SF-006-1"  // Display image
        }
        
        component "Memory card" {
            id: "LC-001-7"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: "SF-004-4"  // Store image
        }
        
        component Flash {
            id: "LC-001-8"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: "SF-002-3"  // Generate Flash
        }
        
        component "User Controls" {
            id: "LC-001-9"
            type: "Hardware"
            color: "#9DC3E6"
            
            allocates: ["SF-006-2", "SF-006-3"]  // Capture shutter/parameters
        }
        
        component "Image Controller" {
            id: "LC-001-10"
            type: "Software"
            color: "#5B9BD5"
            
            allocates: ["SF-004-2", "SF-004-3"]  // Control flash/shooting
        }
        
        // Component ports (connection points)
        port energy {
            direction: OUT
            provides: "IPower"
        }
        
        port image {
            direction: OUT
            provides: "IImageData"
        }
        
        port manual_action {
            direction: IN
            requires: "IUserInput"
        }
    }
    
    // External components
    component User {
        id: "LC-EXT-001"
        type: "External"
        color: "#5B9BD5"
    }
    
    component Environment {
        id: "LC-EXT-002"
        type: "External"
        color: "#5B9BD5"
    }
    
    // Component exchanges (labeled connectors)
    component_exchange "User Input" {
        from: "LC-EXT-001"
        to: "LC-001-9"
        exchange_item: "Manual_Action"
        label: "manual action"
    }
    
    component_exchange "Image Display" {
        from: "LC-001-6"
        to: "LC-EXT-001"
        exchange_item: "Image"
        label: "image"
    }
    
    component_exchange "Light Reflection" {
        from: "LC-EXT-002"
        to: "LC-001-5"
        exchange_item: "Reflected_Light"
        label: "reflected light"
    }
    
    component_exchange "Flash Light" {
        from: "LC-001-8"
        to: "LC-EXT-002"
        exchange_item: "Light"
        label: "light"
    }
    
    // Internal exchanges
    component_exchange "Shooting Request" {
        from: "LC-001-9"
        to: "LC-001-10"
        exchange_item: "Shooting_Command"
        label: "shooting request"
    }
    
    component_exchange "Flash Command" {
        from: "LC-001-10"
        to: "LC-001-8"
        exchange_item: "Flash_Command"
        label: "flash cmd"
    }
    
    component_exchange "Shooting Command" {
        from: "LC-001-10"
        to: "LC-001-4"
        exchange_item: "Shooting_Command"
        label: "shooting cmd"
    }
    
    component_exchange "Optical Image" {
        from: "LC-001-5"
        to: "LC-001-4"
        exchange_item: "Optical_Image"
        label: "optical image"
    }
    
    component_exchange "Raw Image" {
        from: "LC-001-4"
        to: "LC-001-3"
        exchange_item: "Image_Raw"
        label: "raw image"
    }
    
    component_exchange "Processed Image" {
        from: "LC-001-3"
        to: "LC-001-7"
        exchange_item: "Image_Processed"
        label: "processed image"
    }
    
    component_exchange "Stored Image" {
        from: "LC-001-7"
        to: "LC-001-6"
        exchange_item: "Image"
        label: "image"
    }
    
    component_exchange "Energy" {
        from: "LC-001-2"
        to: "LC-001-3"
        exchange_item: "Power"
        label: "energy"
    }
    
    // Dashed green boxes for unallocated functions
    unallocated_function "SF-001" {
        reason: "Environmental function - not allocated to system components"
        display_style: "dashed_green"
    }
}
```

**Renderer with Nested Components:**

```typescript
export class ComponentBlockDiagramRenderer {
    private renderComponent(component: LogicalComponent, depth: number = 0): SVGGElement {
        const group = createGroup()
        const padding = 15
        
        // Calculate size based on sub-components
        const size = this.calculateComponentSize(component)
        
        // Component box (blue rectangle)
        const rect = createRect({
            x: component.x,
            y: component.y,
            width: size.width,
            height: size.height,
            rx: 5,
            fill: component.color,
            stroke: '#2F5597',
            strokeWidth: 2.5
        })
        group.appendChild(rect)
        
        // Component icon (top-left corner)
        const icon = this.getComponentIcon(component.type)
        icon.setAttribute('transform', `translate(${component.x + 10}, ${component.y + 10})`)
        group.appendChild(icon)
        
        // Component name (below icon)
        const name = createText({
            x: component.x + padding,
            y: component.y + 45,
            text: component.name,
            fontSize: 13,
            fontWeight: 'bold',
            fill: 'white'
        })
        group.appendChild(name)
        
        // Render sub-components (nested inside)
        let subY = component.y + 60
        for (const sub of component.subComponents) {
            sub.x = component.x + padding
            sub.y = subY
            
            const subGroup = this.renderComponent(sub, depth + 1)
            group.appendChild(subGroup)
            
            subY += this.calculateComponentSize(sub).height + 10
        }
        
        // Render allocated functions (green boxes inside component)
        for (const funcId of component.allocatedFunctions) {
            const func = this.findFunction(funcId)
            const allocBox = this.renderAllocatedFunction(func, component)
            group.appendChild(allocBox)
        }
        
        // Render component ports
        for (const port of component.ports) {
            const portElement = this.renderComponentPort(port, component)
            group.appendChild(portElement)
        }
        
        return group
    }
    
    private renderAllocatedFunction(func: SystemFunction, parent: LogicalComponent): SVGGElement {
        const group = createGroup()
        const width = 140
        const height = 50
        
        // Position inside parent component
        const x = parent.x + parent.width - width - 15
        const y = parent.y + 70
        
        // Green box (Capella style for allocated functions)
        const rect = createRect({
            x,
            y,
            width,
            height,
            rx: 8,
            fill: '#A9D18E',
            stroke: '#70AD47',
            strokeWidth: 2
        })
        group.appendChild(rect)
        
        // Function name
        const name = createText({
            x: x + width / 2,
            y: y + height / 2,
            text: func.name,
            fontSize: 10,
            textAnchor: 'middle',
            fontWeight: 'bold'
        })
        group.appendChild(name)
        
        return group
    }
    
    private renderUnallocatedFunction(funcId: string): SVGGElement {
        const group = createGroup()
        const func = this.findFunction(funcId)
        
        // Dashed green box (not yet allocated)
        const rect = createRect({
            x: func.x,
            y: func.y,
            width: 150,
            height: 60,
            rx: 8,
            fill: 'none',
            stroke: '#00FF00',
            strokeWidth: 2.5,
            strokeDasharray: '8,4'
        })
        group.appendChild(rect)
        
        // Warning icon
        const warning = createText({
            x: func.x + 10,
            y: func.y + 20,
            text: '⚠️',
            fontSize: 16
        })
        group.appendChild(warning)
        
        // Function name
        const name = createText({
            x: func.x + 35,
            y: func.y + 35,
            text: func.name,
            fontSize: 11,
            fill: '#00AA00',
            fontWeight: 'bold'
        })
        group.appendChild(name)
        
        return group
    }
}
```

### Week 19-20: Component Ports & Interfaces

### Week 21-22: Data Exchanges with Type Information

---

## PHASE 5: Physical Layer (Weeks 23-26)

### Week 23-24: Physical Node Diagrams

**Complete IFE Physical Architecture:**

```arc
physical_architecture "IFE Physical Deployment" {
    // Physical node (yellow box - hardware)
    node "Private Video Display Unit" {
        id: "PN-001"
        type: hardware
        color: "#FFE699"
        processor: "ARM Cortex-A53"
        memory: "2GB RAM"
        
        // Behavior components (blue nested boxes - software)
        behavior_component "PVDU Screen SW" {
            id: "BC-001"
            allocates: "LF-015"  // Display function
            color: "#5B9BD5"
        }
        
        behavior_component "PTV Video Player" {
            id: "BC-002"
            allocates: "LF-010"  // Video playback
            color: "#5B9BD5"
        }
        
        behavior_component "PVDU Video Decoder" {
            id: "BC-003"
            allocates: "LF-008"  // Video decoding
            color: "#5B9BD5"
        }
        
        behavior_component "PVDU Audio Decoder" {
            id: "BC-004"
            allocates: "LF-009"  // Audio decoding
            color: "#5B9BD5"
        }
        
        behavior_component "DAC SW" {
            id: "BC-005"
            allocates: "LF-011"  // Digital-to-Analog conversion
            color: "#5B9BD5"
        }
        
        // Hardware components (gray boxes)
        hardware_component "PVDU Screen" {
            id: "HW-001"
            type: "Display"
            color: "#D0CECE"
        }
        
        hardware_component "PVDU Processor" {
            id: "HW-002"
            type: "CPU"
            specs: "ARM Cortex-A53 @ 1.2GHz"
            color: "#D0CECE"
        }
        
        hardware_component "PVDU MPEG Decoder" {
            id: "HW-003"
            type: "DSP"
            color: "#D0CECE"
        }
        
        hardware_component "Digital Analog Converter" {
            id: "HW-004"
            type: "DAC"
            color: "#D0CECE"
        }
    }
    
    node "IFE Server" {
        id: "PN-002"
        type: hardware
        color: "#FFE699"
        
        behavior_component "Content Manager" {
            id: "BC-006"
            allocates: "LF-001"
            color: "#5B9BD5"
        }
        
        behavior_component "Streaming Server" {
            id: "BC-007"
            allocates: "LF-002"
            color: "#5B9BD5"
        }
    }
    
    // Physical links (blue arrows with protocol labels)
    physical_link "HDMI Connection" {
        from: "HW-002"
        to: "HW-001"
        protocol: "HDMI"
        bandwidth: "1.4 Gbps"
        color: "#4472C4"
    }
    
    physical_link "PVDU BUS" {
        from: "PN-002"
        to: "PN-001"
        protocol: "Ethernet"
        bandwidth: "100 Mbps"
        color: "#4472C4"
    }
    
    // Physical exchanges (data transmission over links)
    physical_exchange "Video Stream" {
        from: "BC-007"  // Streaming Server
        to: "BC-003"    // PVDU Video Decoder
        via: "PVDU BUS"
        message_type: "Packetized Video Packets"
        frequency: "30 fps"
        label: "Packetized Video Packets"
    }
    
    physical_exchange "Decoded Video" {
        from: "BC-003"  // Video Decoder
        to: "BC-002"    // Video Player
        message_type: "Frame Buffer"
        label: "Packet Transmission"
    }
    
    physical_exchange "Display Signal" {
        from: "BC-001"  // Screen SW
        to: "HW-001"    // Screen
        via: "HDMI Connection"
        message_type: "HDMI Signal"
        label: "HDMI"
    }
}
```

**Renderer:**

```typescript
export class PhysicalNodeRenderer {
    private renderNode(node: PhysicalNode): SVGGElement {
        const group = createGroup()
        
        // Node box (yellow for hardware)
        const rect = createRect({
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.height,
            rx: 8,
            fill: '#FFE699',
            stroke: '#BF8F00',
            strokeWidth: 3
        })
        group.appendChild(rect)
        
        // Node header (darker yellow strip)
        const header = createRect({
            x: node.x,
            y: node.y,
            width: node.width,
            height: 40,
            rx: 8,
            fill: '#FFC000',
            stroke: 'none'
        })
        group.appendChild(header)
        
        // Node name
        const name = createText({
            x: node.x + 15,
            y: node.y + 25,
            text: node.name,
            fontSize: 14,
            fontWeight: 'bold',
            fill: '#333'
        })
        group.appendChild(name)
        
        // Hardware specs (small text)
        if (node.hardwareSpecs) {
            const specs = createText({
                x: node.x + 15,
                y: node.y + 55,
                text: `${node.hardwareSpecs.processor} | ${node.hardwareSpecs.memory}`,
                fontSize: 9,
                fill: '#666',
                fontStyle: 'italic'
            })
            group.appendChild(specs)
        }
        
        // Render behavior components (blue nested boxes)
        let subY = node.y + 70
        for (const bc of node.behaviorComponents) {
            const bcGroup = this.renderBehaviorComponent(bc, node.x + 15, subY)
            group.appendChild(bcGroup)
            subY += 60
        }
        
        // Render hardware components (gray boxes)
        for (const hw of node.hardwareComponents) {
            const hwGroup = this.renderHardwareComponent(hw, node.x + node.width - 170, subY)
            group.appendChild(hwGroup)
            subY += 55
        }
        
        return group
    }
    
    private renderBehaviorComponent(bc: BehaviorComponent, x: number, y: number): SVGGElement {
        const group = createGroup()
        
        // Blue box (software/behavior)
        const rect = createRect({
            x,
            y,
            width: 180,
            height: 50,
            rx: 5,
            fill: '#5B9BD5',
            stroke: '#2F5597',
            strokeWidth: 2
        })
        group.appendChild(rect)
        
        // SW icon
        const icon = this.getSoftwareIcon()
        icon.setAttribute('transform', `translate(${x + 8}, ${y + 8})`)
        group.appendChild(icon)
        
        // Component name (white text)
        const name = createText({
            x: x + 40,
            y: y + 28,
            text: bc.name,
            fontSize: 11,
            fill: 'white',
            fontWeight: 'bold'
        })
        group.appendChild(name)
        
        // Allocated functions (green boxes inside)
        if (bc.allocatedFunctions.length > 0) {
            const funcBox = createRect({
                x: x + 5,
                y: y + 35,
                width: 80,
                height: 12,
                rx: 3,
                fill: '#A9D18E',
                stroke: '#70AD47',
                strokeWidth: 1
            })
            group.appendChild(funcBox)
            
            const funcText = createText({
                x: x + 45,
                y: y + 43,
                text: bc.allocatedFunctions[0],
                fontSize: 8,
                textAnchor: 'middle'
            })
            group.appendChild(funcText)
        }
        
        return group
    }
    
    private renderHardwareComponent(hw: HardwareComponent, x: number, y: number): SVGGElement {
        const group = createGroup()
        
        // Gray box (hardware)
        const rect = createRect({
            x,
            y,
            width: 150,
            height: 45,
            rx: 4,
            fill: '#D0CECE',
            stroke: '#7F7F7F',
            strokeWidth: 2
        })
        group.appendChild(rect)
        
        // HW icon
        const icon = this.getHardwareIcon(hw.type)
        icon.setAttribute('transform', `translate(${x + 8}, ${y + 8})`)
        group.appendChild(icon)
        
        // Component name
        const name = createText({
            x: x + 35,
            y: y + 22,
            text: hw.name,
            fontSize: 10,
            fontWeight: 'bold'
        })
        group.appendChild(name)
        
        // Specs (if available)
        if (hw.specs) {
            const specs = createText({
                x: x + 35,
                y: y + 35,
                text: hw.specs,
                fontSize: 8,
                fill: '#666'
            })
            group.appendChild(specs)
        }
        
        return group
    }
    
    private renderPhysicalLink(link: PhysicalLink): SVGGElement {
        const group = createGroup()
        
        const source = this.getNodePosition(link.from)
        const target = this.getNodePosition(link.to)
        
        // Thick blue arrow (Capella style for physical links)
        const arrow = createPath({
            d: this.createOrthogonalPath(source, target),
            fill: 'none',
            stroke: link.color || '#4472C4',
            strokeWidth: 4,
            markerEnd: 'url(#physical-arrowhead)'
        })
        group.appendChild(arrow)
        
        // Protocol label (white text on blue background)
        const midPoint = this.getMidPoint(source, target)
        const labelBg = createRect({
            x: midPoint.x - 50,
            y: midPoint.y - 12,
            width: 100,
            height: 22,
            rx: 4,
            fill: '#4472C4',
            stroke: '#2F5597',
            strokeWidth: 1.5
        })
        group.appendChild(labelBg)
        
        const protocolText = createText({
            x: midPoint.x,
            y: midPoint.y + 4,
            text: link.protocol,
            fontSize: 11,
            fontWeight: 'bold',
            fill: 'white',
            textAnchor: 'middle'
        })
        group.appendChild(protocolText)
        
        // Bandwidth (smaller text below)
        if (link.bandwidth) {
            const bandwidth = createText({
                x: midPoint.x,
                y: midPoint.y + 25,
                text: link.bandwidth,
                fontSize: 9,
                fill: '#666',
                textAnchor: 'middle',
                fontStyle: 'italic'
            })
            group.appendChild(bandwidth)
        }
        
        return group
    }
}
```

### Week 25-26: Allocation Visualization & Deployment

---

## PHASE 6: Behavioral (Weeks 27-30)

### Week 27-28: Sequence Diagrams

**(Implementation already detailed in previous sections)**

### Week 29-30: State Machine Diagrams

**Complete Example:**

```arc
component "Rover Controller" {
    id: "LC-005"
    
    state_machine "Rover Control FSM" {
        initial_state: "Manual drive"
        
        // States with entry/exit actions
        state "Manual drive" {
            entry_action: "Start video"
            color: "#BDD7EE"
        }
        
        state "Calibrating" {
            entry_action: "Calibrate Rover"
            color: "#FFE699"
        }
        
        state "Stopped" {
            color: "#F4B184"
        }
        
        state "Automated drive" {
            color: "#C5E0B4"
        }
        
        state "Park assist engaged" {
            entry_action: "Activate Park Procedure"
            entry_action: "Start video"
            color: "#A9D18E"
        }
        
        state "Obstacle avoidance engaged" {
            entry_action: "Implement self protection"
            color: "#FF9999"
        }
        
        // Transitions with triggers and guards
        transition {
            from: initial
            to: "Manual drive"
            trigger: "Init"
        }
        
        transition {
            from: "Manual drive"
            to: "Calibrating"
            trigger: "calibration start/stop"
            guard: "[START]"
        }
        
        transition {
            from: "Calibrating"
            to: "Stopped"
            trigger: "calibration start/stop"
            guard: "[STOP]"
        }
        
        transition {
            from: "Stopped"
            to: "Manual drive"
            trigger: "joystick state change"
            guard: "[NON NEUTRAL]"
        }
        
        transition {
            from: "Automated drive"
            to: "Manual drive"
            trigger: "joystick state change"
            guard: "[JOYSTICK NEUTRAL]"
        }
        
        transition {
            from: "Park assist engaged"
            to: "Stopped"
            trigger: "Abort"
        }
        
        transition {
            from: "Automated drive"
            to: "Obstacle avoidance engaged"
            trigger: "obstacle high proximity warning"
        }
        
        transition {
            from: "Obstacle avoidance engaged"
            to: "Automated drive"
            trigger: "obstacle clear"
        }
        
        transition {
            from: "Manual drive"
            to: "Park assist engaged"
            trigger: "PA start/stop"
        }
        
        transition {
            from: "Park assist engaged"
            to: "Manual drive"
            trigger: "PA start/stop"
        }
    }
}
```

**State Machine Renderer:**

```typescript
export class StateMachineRenderer {
    private STATE_WIDTH = 180
    private STATE_HEIGHT = 80
    
    private renderState(state: State): SVGGElement {
        const group = createGroup()
        
        // State box (rounded rectangle with colored background)
        const rect = createRect({
            x: state.x,
            y: state.y,
            width: this.STATE_WIDTH,
            height: this.STATE_HEIGHT,
            rx: 15,
            ry: 15,
            fill: state.color || '#E0E0E0',
            stroke: '#666',
            strokeWidth: 2.5
        })
        group.appendChild(rect)
        
        // State name (bold text at top)
        const name = createText({
            x: state.x + this.STATE_WIDTH / 2,
            y: state.y + 25,
            text: state.name,
            fontSize: 13,
            fontWeight: 'bold',
            textAnchor: 'middle'
        })
        group.appendChild(name)
        
        // Divider line
        const divider = createLine({
            x1: state.x + 10,
            y1: state.y + 35,
            x2: state.x + this.STATE_WIDTH - 10,
            y2: state.y + 35,
            stroke: '#666',
            strokeWidth: 1
        })
        group.appendChild(divider)
        
        // Entry/Exit actions (below divider)
        let actionY = state.y + 50
        for (const action of state.entryActions) {
            const actionText = createText({
                x: state.x + 10,
                y: actionY,
                text: `entry / ${action}`,
                fontSize: 10,
                fontStyle: 'italic'
            })
            group.appendChild(actionText)
            actionY += 15
        }
        
        for (const action of state.exitActions) {
            const actionText = createText({
                x: state.x + 10,
                y: actionY,
                text: `exit / ${action}`,
                fontSize: 10,
                fontStyle: 'italic'
            })
            group.appendChild(actionText)
            actionY += 15
        }
        
        return group
    }
    
    private renderTransition(transition: Transition): SVGGElement {
        const group = createGroup()
        
        const source = this.getStatePosition(transition.from)
        const target = this.getStatePosition(transition.to)
        
        // Curved arrow
        const path = this.createCurvedPath(source, target)
        const arrow = createPath({
            d: path,
            fill: 'none',
            stroke: '#333',
            strokeWidth: 2,
            markerEnd: 'url(#state-arrowhead)'
        })
        group.appendChild(arrow)
        
        // Transition label: trigger [guard] / action
        const midPoint = this.getMidPoint(source, target)
        let label = transition.trigger
        if (transition.guard) {
            label += ` ${transition.guard}`
        }
        if (transition.action) {
            label += ` / ${transition.action}`
        }
        
        const labelBg = createRect({
            x: midPoint.x - 60,
            y: midPoint.y - 10,
            width: 120,
            height: 18,
            rx: 3,
            fill: 'white',
            stroke: '#333',
            strokeWidth: 1
        })
        group.appendChild(labelBg)
        
        const labelText = createText({
            x: midPoint.x,
            y: midPoint.y + 3,
            text: label,
            fontSize: 9,
            textAnchor: 'middle'
        })
        group.appendChild(labelText)
        
        return group
    }
    
    private renderInitialState(x: number, y: number): SVGGElement {
        const group = createGroup()
        
        // Black filled circle
        const circle = createCircle({
            cx: x,
            cy: y,
            r: 12,
            fill: '#000',
            stroke: '#000',
            strokeWidth: 2
        })
        group.appendChild(circle)
        
        return group
    }
}
```

---

## PHASE 7: Polish & Export (Weeks 31-32)

### Week 31: Interactive Features

```typescript
// apps/web/components/diagrams/interactive-diagram.tsx
export function InteractiveDiagram({ svg, type }: { svg: string, type: DiagramType }) {
    const [zoom, setZoom] = useState(1)
    const [pan, setPan] = useState({ x: 0, y: 0 })
    const [selectedElement, setSelectedElement] = useState<string | null>(null)
    
    return (
        <div className="diagram-container">
            <div className="diagram-toolbar">
                <Button onClick={() => setZoom(z => z * 1.2)}>
                    <ZoomIn className="h-4 w-4" />
                </Button>
                <Button onClick={() => setZoom(z => z / 1.2)}>
                    <ZoomOut className="h-4 w-4" />
                </Button>
                <Button onClick={handleExportPNG}>
                    <Download className="h-4 w-4" /> PNG
                </Button>
                <Button onClick={handleExportSVG}>
                    <Download className="h-4 w-4" /> SVG
                </Button>
                <Button onClick={handleExportPDF}>
                    <Download className="h-4 w-4" /> PDF
                </Button>
            </div>
            
            <div
                className="diagram-viewport"
                onWheel={handleWheel}
                onMouseDown={handlePanStart}
                onMouseMove={handlePanMove}
            >
                <svg
                    dangerouslySetInnerHTML={{ __html: svg }}
                    style={{
                        transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`
                    }}
                    onClick={handleElementClick}
                />
            </div>
            
            {selectedElement && (
                <ElementProperties element={selectedElement} />
            )}
        </div>
    )
}
```

### Week 32: Export & Documentation

```bash
# Export all diagram types
arclang export model.arc --diagram operational -o operational.svg
arclang export model.arc --diagram functional -o functional.svg
arclang export model.arc --diagram logical -o logical.svg
arclang export model.arc --diagram physical -o physical.svg
arclang export model.arc --diagram sequence -o sequence.svg
arclang export model.arc --diagram state -o state.svg

# PDF report with all diagrams
arclang export model.arc --format pdf --all-diagrams -o complete-architecture.pdf
```

---

## Success Metrics

- ✅ **10 Diagram Types**: OAD, Capability, Functional, Logical, Physical, Sequence, State, Data Model, Hierarchy, Interfaces
- ✅ **Visual Fidelity**: 95% match to Capella appearance
- ✅ **Rendering Speed**: < 1 second for typical diagrams
- ✅ **Export Formats**: PNG, SVG, PDF
- ✅ **Interactive**: Pan, zoom, click, hover
- ✅ **Traceability**: Full model traceability across layers

---

## Timeline Summary

**32 weeks (8 months) to complete Capella-quality diagram system**

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Foundation | 4 weeks | Syntax + Parser |
| 2. Operational | 6 weeks | OAD + Capabilities |
| 3. System | 6 weeks | Functional + Hierarchy |
| 4. Logical | 6 weeks | Components + Allocation |
| 5. Physical | 4 weeks | Nodes + Deployment |
| 6. Behavioral | 4 weeks | Sequence + State Machine |
| 7. Polish | 2 weeks | Export + UX |

---

**Ready to begin Phase 1 next week?** 🚀
