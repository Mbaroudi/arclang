# Complete Capella-Quality Diagram System for ArcLang

## Vision Statement

Build a **world-class MBSE visualization system** that matches or exceeds Capella's diagram capabilities, with support for:
- Operational Activity Diagrams (OAD)
- Capability Decomposition
- Functional Architecture Diagrams
- Physical Architecture Diagrams
- Sequence Diagrams with advanced features
- State Machine Diagrams
- Custom layouts beyond ELK

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    ArcLang Compiler                          │
│  (Rust) - Parses .arc files → Semantic Model                │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│              Diagram Generator Service                       │
│  (Node.js/TypeScript) - Converts model → Diagram Data       │
├─────────────────────────────────────────────────────────────┤
│  ├─ OAD Renderer (Swimlanes, Activities, Actors)           │
│  ├─ Capability Renderer (Hierarchical Boxes)               │
│  ├─ Functional Renderer (Functions, Ports, Data Flows)     │
│  ├─ Physical Renderer (Nodes, Links, Allocation)           │
│  ├─ Sequence Renderer (Lifelines, Messages, Fragments)     │
│  └─ State Machine Renderer (States, Transitions)           │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│           Advanced Layout Engines                            │
├─────────────────────────────────────────────────────────────┤
│  ├─ ELK (Hierarchical, Orthogonal)                         │
│  ├─ Dagre (Layered Graphs)                                 │
│  ├─ Custom Swimlane Layout                                 │
│  ├─ Custom Timeline Layout (for Sequence)                  │
│  └─ Force-Directed Layout (for Networks)                   │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│        Interactive Diagram Renderer (React)                  │
│  - SVG/Canvas rendering with pan/zoom                       │
│  - Interactive elements (click, hover, edit)                │
│  - Export (PNG, SVG, PDF)                                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Operational Activity Diagram (OAD)

### Features to Implement

#### 1.1 Swimlanes
- **Horizontal lanes** for each actor/entity
- **Vertical separation** between operational entities
- **Resizable lanes** based on content
- **Lane headers** with actor icons/names

#### 1.2 Activities
- **Yellow/Golden boxes** (Capella standard)
- **Rounded corners**
- **Activity icons** (circular icons inside)
- **Hierarchical activities** (parent-child relationships)
- **Activity delegation** (shows which actor performs)

#### 1.3 Data Flows
- **Labeled arrows** between activities
- **Flow direction** (left-to-right, top-to-bottom)
- **Data types** on arrows
- **Multiple flows** between activities

### ArcLang Syntax for OAD

```arc
operational_analysis "In-Flight Entertainment" {
    // Define operational entities (create swimlanes)
    entity "Passenger" {
        id: "OE-001"
        type: actor
        icon: "person"
    }
    
    entity "IFE System" {
        id: "OE-002"
        type: system
        icon: "monitor"
    }
    
    entity "Aircraft Systems" {
        id: "OE-003"
        type: system
        icon: "airplane"
    }
    
    // Operational activities
    operational_activity "Listen to Audio" {
        id: "OA-001"
        performed_by: "OE-001"  // Passenger
        category: "entertainment"
        icon: "headphones"
        color: "#FFD966"  // Yellow
        
        // Sub-activities
        sub_activity "Select Audio Channel" {
            id: "OA-001-1"
        }
        
        sub_activity "Adjust Volume" {
            id: "OA-001-2"
        }
    }
    
    operational_activity "Provide Audio Content" {
        id: "OA-002"
        performed_by: "OE-002"  // IFE System
        category: "service"
    }
    
    operational_activity "Broadcast Announcements" {
        id: "OA-003"
        performed_by: "OE-003"  // Aircraft
        category: "communication"
    }
    
    // Data flows
    operational_exchange "Audio Selection" {
        from: "OA-001"
        to: "OA-002"
        data_type: "Channel_Selection"
        protocol: "User Input"
    }
    
    operational_exchange "Audio Stream" {
        from: "OA-002"
        to: "OA-001"
        data_type: "Audio_Data"
        protocol: "Streaming"
    }
    
    operational_exchange "Announcement" {
        from: "OA-003"
        to: "OA-002"
        data_type: "Voice_Message"
        protocol: "PA System"
    }
}
```

### Rendering Implementation

```typescript
// apps/web/lib/renderers/operational-activity-renderer.ts
export class OperationalActivityRenderer {
    private layout: SwimlaneLayout
    
    constructor(private model: OperationalAnalysis) {
        this.layout = new SwimlaneLayout()
    }
    
    render(): SVGElement {
        const svg = createSVG()
        
        // 1. Create swimlanes
        const lanes = this.createSwimlanes()
        svg.append(...lanes)
        
        // 2. Place activities in lanes
        const activities = this.createActivities()
        svg.append(...activities)
        
        // 3. Draw data flows
        const flows = this.createDataFlows()
        svg.append(...flows)
        
        return svg
    }
    
    private createSwimlanes(): SVGGElement[] {
        const lanes: SVGGElement[] = []
        let yOffset = 0
        
        for (const entity of this.model.entities) {
            const lane = createSVGGroup()
            
            // Lane background
            const rect = createRect({
                x: 0,
                y: yOffset,
                width: 1200,
                height: 200,
                fill: entity.type === 'actor' ? '#E8F4F8' : '#F0F0F0',
                stroke: '#999',
                strokeWidth: 2
            })
            lane.appendChild(rect)
            
            // Lane header
            const header = createSVGGroup()
            const headerRect = createRect({
                x: 0,
                y: yOffset,
                width: 150,
                height: 200,
                fill: '#2E75B6',
                stroke: '#1F4E78'
            })
            header.appendChild(headerRect)
            
            // Entity icon
            if (entity.icon === 'person') {
                const icon = this.createPersonIcon(75, yOffset + 80)
                header.appendChild(icon)
            }
            
            // Entity name
            const text = createText({
                x: 75,
                y: yOffset + 180,
                text: entity.name,
                fill: 'white',
                fontSize: 14,
                fontWeight: 'bold',
                textAnchor: 'middle'
            })
            header.appendChild(text)
            
            lane.appendChild(header)
            lanes.push(lane)
            
            yOffset += 200
        }
        
        return lanes
    }
    
    private createActivities(): SVGGElement[] {
        const activities: SVGGElement[] = []
        
        for (const activity of this.model.activities) {
            const group = createSVGGroup()
            
            // Activity box (yellow with rounded corners)
            const rect = createRoundedRect({
                x: activity.x,
                y: activity.y,
                width: 150,
                height: 80,
                rx: 10,
                fill: activity.color || '#FFD966',
                stroke: '#BF9000',
                strokeWidth: 2
            })
            group.appendChild(rect)
            
            // Activity icon (circular)
            const iconCircle = createCircle({
                cx: activity.x + 30,
                cy: activity.y + 25,
                r: 15,
                fill: 'white',
                stroke: '#BF9000',
                strokeWidth: 2
            })
            group.appendChild(iconCircle)
            
            // Icon image/symbol
            const icon = this.getActivityIcon(activity.icon)
            icon.setAttribute('x', String(activity.x + 20))
            icon.setAttribute('y', String(activity.y + 15))
            group.appendChild(icon)
            
            // Activity name
            const text = createText({
                x: activity.x + 75,
                y: activity.y + 45,
                text: this.wrapText(activity.name, 120),
                fontSize: 12,
                textAnchor: 'middle'
            })
            group.appendChild(text)
            
            // Expansion indicator for hierarchical activities
            if (activity.subActivities && activity.subActivities.length > 0) {
                const expandIcon = createExpandIcon(activity.x + 140, activity.y + 70)
                group.appendChild(expandIcon)
            }
            
            activities.push(group)
        }
        
        return activities
    }
    
    private createDataFlows(): SVGGElement[] {
        const flows: SVGGElement[] = []
        
        for (const exchange of this.model.exchanges) {
            const source = this.findActivity(exchange.from)
            const target = this.findActivity(exchange.to)
            
            if (!source || !target) continue
            
            const flow = createSVGGroup()
            
            // Flow path (curved arrow)
            const path = createCurvedPath({
                x1: source.x + 150,
                y1: source.y + 40,
                x2: target.x,
                y2: target.y + 40,
                color: '#4472C4',
                strokeWidth: 2
            })
            flow.appendChild(path)
            
            // Arrowhead
            const arrow = createArrowhead(target.x, target.y + 40, '#4472C4')
            flow.appendChild(arrow)
            
            // Flow label
            const label = createText({
                x: (source.x + target.x) / 2,
                y: (source.y + target.y) / 2 - 10,
                text: exchange.dataType,
                fontSize: 10,
                fill: '#2E75B6',
                fontStyle: 'italic'
            })
            flow.appendChild(label)
            
            flows.push(flow)
        }
        
        return flows
    }
    
    private createPersonIcon(x: number, y: number): SVGGElement {
        const group = createSVGGroup()
        
        // Head
        const head = createCircle({
            cx: x,
            cy: y - 20,
            r: 15,
            fill: 'white',
            stroke: 'white',
            strokeWidth: 2
        })
        group.appendChild(head)
        
        // Body
        const body = createLine({
            x1: x,
            y1: y - 5,
            x2: x,
            y2: y + 20,
            stroke: 'white',
            strokeWidth: 3
        })
        group.appendChild(body)
        
        // Arms
        const leftArm = createLine({
            x1: x,
            y1: y + 5,
            x2: x - 15,
            y2: y + 15,
            stroke: 'white',
            strokeWidth: 3
        })
        const rightArm = createLine({
            x1: x,
            y1: y + 5,
            x2: x + 15,
            y2: y + 15,
            stroke: 'white',
            strokeWidth: 3
        })
        group.appendChild(leftArm)
        group.appendChild(rightArm)
        
        // Legs
        const leftLeg = createLine({
            x1: x,
            y1: y + 20,
            x2: x - 10,
            y2: y + 35,
            stroke: 'white',
            strokeWidth: 3
        })
        const rightLeg = createLine({
            x1: x,
            y1: y + 20,
            x2: x + 10,
            y2: y + 35,
            stroke: 'white',
            strokeWidth: 3
        })
        group.appendChild(leftLeg)
        group.appendChild(rightLeg)
        
        return group
    }
}

// Custom swimlane layout algorithm
class SwimlaneLayout {
    layout(activities: Activity[], entities: Entity[]): LayoutResult {
        // Sort activities by entity
        const grouped = this.groupByEntity(activities, entities)
        
        // Position activities horizontally in each lane
        const positioned = this.positionInLanes(grouped)
        
        // Optimize for minimal crossings
        return this.minimizeCrossings(positioned)
    }
    
    private groupByEntity(activities: Activity[], entities: Entity[]): Map<string, Activity[]> {
        const groups = new Map<string, Activity[]>()
        
        for (const activity of activities) {
            const entityId = activity.performedBy
            if (!groups.has(entityId)) {
                groups.set(entityId, [])
            }
            groups.get(entityId)!.push(activity)
        }
        
        return groups
    }
    
    private positionInLanes(grouped: Map<string, Activity[]>): Activity[] {
        const positioned: Activity[] = []
        let laneIndex = 0
        
        for (const [entityId, activities] of grouped) {
            let xOffset = 200  // Start after lane header
            const yBase = laneIndex * 200 + 60  // Center in lane
            
            for (const activity of activities) {
                activity.x = xOffset
                activity.y = yBase
                positioned.push(activity)
                
                xOffset += 200  // Spacing between activities
            }
            
            laneIndex++
        }
        
        return positioned
    }
    
    private minimizeCrossings(activities: Activity[]): LayoutResult {
        // Apply heuristics to reduce edge crossings
        // Similar to Sugiyama framework for layered graphs
        
        // TODO: Implement crossing minimization
        // For now, return as-is
        
        return {
            activities,
            width: 1200,
            height: activities.length * 200
        }
    }
}
```

---

## 2. Capability Decomposition Diagram

### Features

- **Hierarchical boxes** (parent capabilities contain child capabilities)
- **Color coding** by capability level
- **Capability relationships** (associations, dependencies)
- **Mission/Capability/Sub-capability** hierarchy

### Syntax

```arc
operational_analysis "Mission Analysis" {
    capability "Air Traffic Management" {
        id: "CAP-001"
        level: "mission"
        color: "#4472C4"
        
        capability "Flight Planning" {
            id: "CAP-001-1"
            level: "capability"
            
            capability "Route Calculation" {
                id: "CAP-001-1-1"
                level: "sub-capability"
            }
            
            capability "Weather Integration" {
                id: "CAP-001-1-2"
                level: "sub-capability"
            }
        }
        
        capability "Flight Monitoring" {
            id: "CAP-001-2"
            level: "capability"
        }
    }
    
    // Capability relationships
    capability_association "CAP-001-1" involves "CAP-001-2" {
        type: "collaboration"
    }
}
```

---

## 3. Functional Architecture Diagram

### Features

#### 3.1 Functions
- **Blue boxes** for functions
- **Hierarchical decomposition** (root functions → sub-functions)
- **Function categories** (displayed as colors/icons)
- **Function ports** (input/output)

#### 3.2 Functional Ports
- **Small squares** on function borders
- **Direction indicators** (IN, OUT, IN/OUT)
- **Port types** (data, control, event)
- **Port connections** between functions

#### 3.3 Data Flows
- **Arrows between ports**
- **Data type labels**
- **Flow direction**
- **Multiple connections**

### Syntax

```arc
system_analysis "Brake Control System" {
    // Root function
    system_function "Control Braking" {
        id: "SF-001"
        category: "control"
        color: "#5B9BD5"
        
        // Input ports
        port "BrakeRequest" {
            direction: IN
            type: control
            data_type: "Brake_Command"
        }
        
        port "VehicleSpeed" {
            direction: IN
            type: data
            data_type: "Speed_km_h"
        }
        
        // Output port
        port "BrakeForce" {
            direction: OUT
            type: data
            data_type: "Force_N"
        }
        
        // Sub-functions
        sub_function "Calculate Required Force" {
            id: "SF-001-1"
            category: "computation"
            
            port "SpeedIn" {
                direction: IN
                type: data
            }
            
            port "ForceOut" {
                direction: OUT
                type: data
            }
        }
        
        sub_function "Apply Safety Limits" {
            id: "SF-001-2"
            category: "safety"
            safety_level: "ASIL_D"
            
            port "ForceIn" {
                direction: IN
                type: data
            }
            
            port "LimitedForceOut" {
                direction: OUT
                type: data
            }
        }
    }
    
    // Functional exchanges (port-to-port connections)
    functional_exchange "Speed Data Flow" {
        from: "SF-001.VehicleSpeed"
        to: "SF-001-1.SpeedIn"
        data_type: "Speed_km_h"
    }
    
    functional_exchange "Force Calculation" {
        from: "SF-001-1.ForceOut"
        to: "SF-001-2.ForceIn"
        data_type: "Force_N"
    }
}
```

---

## 4. Physical Architecture Diagram

### Features

#### 4.1 Physical Components
- **Gray boxes** for hardware nodes
- **White boxes** for behavior components (allocated functions)
- **Dashed green boxes** for not-yet-allocated functions
- **Component hierarchy**

#### 4.2 Physical Links
- **Solid lines** for physical connections
- **Protocol labels** (CAN, Ethernet, etc.)
- **Bandwidth indicators**

#### 4.3 Allocation Visualization
- **Functions inside nodes** (shows deployment)
- **Color indicators** for allocation status

### Syntax

```arc
physical_architecture "ECU Architecture" {
    // Hardware nodes
    node "Main ECU" {
        id: "PN-001"
        type: "hardware"
        processor: "Infineon AURIX TC3xx"
        memory: "4MB RAM"
        color: "#808080"
        
        // Allocated behavior components
        behavior_component "Brake Controller" {
            id: "BC-001"
            allocates: "SF-001"  // System function
            safety_level: "ASIL_D"
            color: "#FFFFFF"
        }
        
        behavior_component "Speed Monitor" {
            id: "BC-002"
            allocates: "SF-002"
            safety_level: "ASIL_C"
            color: "#FFFFFF"
        }
    }
    
    node "Sensor ECU" {
        id: "PN-002"
        type: "hardware"
        processor: "STM32F7"
        
        behavior_component "Sensor Interface" {
            id: "BC-003"
            allocates: "SF-003"
            color: "#FFFFFF"
        }
    }
    
    // Unallocated functions (dashed green boxes)
    unallocated_function "Diagnostic Monitor" {
        id: "SF-004"
        status: "not_allocated"
        color: "#00FF00"
        border_style: "dashed"
    }
    
    // Physical links
    physical_link "CAN Bus" {
        from: "PN-001"
        to: "PN-002"
        protocol: "CAN FD"
        bandwidth: "5 Mbps"
        color: "#FF6600"
    }
    
    // Data exchanges over physical links
    physical_exchange "Sensor Data" {
        from: "BC-003"
        to: "BC-002"
        via: "CAN Bus"
        message_type: "Speed_Message"
        frequency: "10ms"
    }
}
```

---

## 5. Sequence Diagram (Advanced Features)

### Features

#### 5.1 Lifelines
- **Vertical dashed lines** for each participant
- **Participant boxes** at top
- **Activation boxes** (white rectangles on lifelines)

#### 5.2 Messages
- **Solid arrows** for synchronous messages
- **Dashed arrows** for asynchronous messages
- **Return messages** (dashed with open arrowhead)
- **Message labels** with parameters

#### 5.3 Combined Fragments
- **PAR (Parallel)**: Multiple interactions simultaneously
- **OPT (Optional)**: Conditional execution
- **LOOP**: Repeated interactions
- **ALT**: Alternative paths

#### 5.4 Timing
- **Time constraints** between messages
- **Duration indicators**

### Syntax

```arc
scenario "Emergency Braking Sequence" {
    participants {
        actor "Driver" {
            id: "ACT-001"
            lifeline_color: "#2E75B6"
        }
        
        component "Radar Sensor" {
            id: "LC-001"
            lifeline_color: "#70AD47"
        }
        
        component "Brake Controller" {
            id: "LC-002"
            lifeline_color: "#FFC000"
        }
        
        component "Brake Actuator" {
            id: "LC-003"
            lifeline_color: "#C00000"
        }
    }
    
    // Message sequence
    sequence {
        // Simple synchronous message
        message "Obstacle Detected" {
            from: "LC-001"
            to: "LC-002"
            type: synchronous
            activation: true  // Shows activation box on LC-002
            timing: "< 5ms"
        }
        
        // Activation box for processing
        activate "LC-002" {
            duration: "10ms"
            
            // Internal processing (self-message)
            message "Calculate Brake Force" {
                from: "LC-002"
                to: "LC-002"
                type: synchronous
            }
        }
        
        // Combined fragment: PAR (parallel execution)
        fragment PAR {
            label: "Parallel Actions"
            
            operand "Warning to Driver" {
                message "Visual Alert" {
                    from: "LC-002"
                    to: "ACT-001"
                    type: asynchronous
                }
                
                message "Audible Alert" {
                    from: "LC-002"
                    to: "ACT-001"
                    type: asynchronous
                }
            }
            
            operand "Brake Application" {
                message "Apply Brakes" {
                    from: "LC-002"
                    to: "LC-003"
                    type: synchronous
                    params: "force=800N"
                    timing: "< 2ms"
                }
                
                // Return message
                message "Brakes Applied" {
                    from: "LC-003"
                    to: "LC-002"
                    type: return
                }
            }
        }
        
        // Combined fragment: OPT (optional execution)
        fragment OPT {
            label: "If Driver Overrides"
            condition: "driver_input == true"
            
            message "Override Command" {
                from: "ACT-001"
                to: "LC-002"
                type: synchronous
            }
            
            message "Release Brakes" {
                from: "LC-002"
                to: "LC-003"
                type: synchronous
            }
        }
        
        // Combined fragment: LOOP
        fragment LOOP {
            label: "Monitor Until Stop"
            condition: "speed > 0"
            
            message "Check Speed" {
                from: "LC-002"
                to: "LC-001"
                type: synchronous
            }
            
            message "Current Speed" {
                from: "LC-001"
                to: "LC-002"
                type: return
                params: "speed=45 km/h"
            }
        }
        
        deactivate "LC-002"
    }
    
    // Time constraints
    timing_constraint "End-to-End Latency" {
        from: "Obstacle Detected"
        to: "Brakes Applied"
        max_duration: "20ms"
        requirement: "SYS-001"
    }
}
```

### Rendering Implementation

```typescript
// apps/web/lib/renderers/sequence-diagram-renderer.ts
export class SequenceDiagramRenderer {
    private timelineLayout: TimelineLayout
    private currentY: number = 100
    
    render(scenario: Scenario): SVGElement {
        const svg = createSVG()
        
        // 1. Create participant boxes and lifelines
        const lifelines = this.createLifelines(scenario.participants)
        svg.append(...lifelines)
        
        // 2. Render messages in order
        for (const message of scenario.sequence) {
            if (message.type === 'message') {
                const arrow = this.createMessage(message)
                svg.appendChild(arrow)
            } else if (message.type === 'fragment') {
                const fragment = this.createCombinedFragment(message)
                svg.appendChild(fragment)
            } else if (message.type === 'activate') {
                const activation = this.createActivationBox(message)
                svg.appendChild(activation)
            }
        }
        
        return svg
    }
    
    private createLifelines(participants: Participant[]): SVGGElement[] {
        const lifelines: SVGGElement[] = []
        const spacing = 200
        let x = 100
        
        for (const participant of participants) {
            const group = createSVGGroup()
            
            // Participant box
            const box = createRoundedRect({
                x: x - 60,
                y: 20,
                width: 120,
                height: 50,
                rx: 5,
                fill: participant.color || '#FFFFFF',
                stroke: '#000',
                strokeWidth: 2
            })
            group.appendChild(box)
            
            // Participant name
            const text = createText({
                x: x,
                y: 50,
                text: participant.name,
                fontSize: 12,
                textAnchor: 'middle',
                fontWeight: 'bold'
            })
            group.appendChild(text)
            
            // Lifeline (dashed vertical line)
            const lifeline = createLine({
                x1: x,
                y1: 70,
                x2: x,
                y2: 1000,  // Will adjust based on content
                stroke: '#999',
                strokeWidth: 1,
                strokeDasharray: '5,5'
            })
            group.appendChild(lifeline)
            
            // Store position for later
            participant._x = x
            
            lifelines.push(group)
            x += spacing
        }
        
        return lifelines
    }
    
    private createMessage(message: Message): SVGGElement {
        const group = createSVGGroup()
        const source = this.findParticipant(message.from)
        const target = this.findParticipant(message.to)
        
        const y = this.currentY
        
        // Message arrow
        const arrow = message.type === 'synchronous'
            ? this.createSynchronousArrow(source._x, y, target._x, y)
            : this.createAsynchronousArrow(source._x, y, target._x, y)
        group.appendChild(arrow)
        
        // Message label
        const label = createText({
            x: (source._x + target._x) / 2,
            y: y - 5,
            text: message.label,
            fontSize: 11,
            textAnchor: 'middle'
        })
        group.appendChild(label)
        
        // Timing constraint
        if (message.timing) {
            const timing = createText({
                x: (source._x + target._x) / 2,
                y: y + 15,
                text: `{${message.timing}}`,
                fontSize: 9,
                fill: '#666',
                textAnchor: 'middle',
                fontStyle: 'italic'
            })
            group.appendChild(timing)
        }
        
        this.currentY += 40
        return group
    }
    
    private createCombinedFragment(fragment: CombinedFragment): SVGGElement {
        const group = createSVGGroup()
        const startY = this.currentY
        
        // Fragment frame (rounded rectangle)
        const frameWidth = 600
        const frameHeight = fragment.operands.length * 100 + 40
        
        const frame = createRoundedRect({
            x: 50,
            y: startY,
            width: frameWidth,
            height: frameHeight,
            rx: 5,
            fill: 'none',
            stroke: '#000',
            strokeWidth: 2
        })
        group.appendChild(frame)
        
        // Fragment operator label (PAR, OPT, LOOP, ALT)
        const operatorBox = createPolygon({
            points: [[50, startY], [100, startY], [110, startY + 20], [50, startY + 20]],
            fill: '#E0E0E0',
            stroke: '#000',
            strokeWidth: 2
        })
        group.appendChild(operatorBox)
        
        const operatorText = createText({
            x: 70,
            y: startY + 14,
            text: fragment.operator,  // PAR, OPT, etc.
            fontSize: 12,
            fontWeight: 'bold'
        })
        group.appendChild(operatorText)
        
        // Condition (for OPT, LOOP)
        if (fragment.condition) {
            const condition = createText({
                x: 120,
                y: startY + 14,
                text: `[${fragment.condition}]`,
                fontSize: 10,
                fontStyle: 'italic',
                fill: '#666'
            })
            group.appendChild(condition)
        }
        
        // Render operands
        this.currentY += 30
        
        if (fragment.operator === 'PAR') {
            // Parallel operands (side by side, separated by dashed line)
            const operandHeight = frameHeight - 40
            const divider = createLine({
                x1: 50 + frameWidth / 2,
                y1: startY + 25,
                x2: 50 + frameWidth / 2,
                y2: startY + frameHeight,
                stroke: '#999',
                strokeWidth: 1,
                strokeDasharray: '5,5'
            })
            group.appendChild(divider)
        } else {
            // Sequential operands (stacked, separated by horizontal dashed lines)
            for (let i = 1; i < fragment.operands.length; i++) {
                const divider = createLine({
                    x1: 50,
                    y1: startY + 25 + (i * 100),
                    x2: 50 + frameWidth,
                    y2: startY + 25 + (i * 100),
                    stroke: '#999',
                    strokeWidth: 1,
                    strokeDasharray: '5,5'
                })
                group.appendChild(divider)
            }
        }
        
        // Render messages inside operands
        for (const operand of fragment.operands) {
            for (const message of operand.messages) {
                const msgElement = this.createMessage(message)
                group.appendChild(msgElement)
            }
        }
        
        this.currentY = startY + frameHeight + 20
        return group
    }
    
    private createActivationBox(activation: Activation): SVGGElement {
        const group = createSVGGroup()
        const participant = this.findParticipant(activation.participantId)
        
        const startY = this.currentY
        const height = activation.duration * 2  // Scale factor
        
        // White rectangle on lifeline
        const box = createRect({
            x: participant._x - 8,
            y: startY,
            width: 16,
            height: height,
            fill: '#FFFFFF',
            stroke: '#000',
            strokeWidth: 1.5
        })
        group.appendChild(box)
        
        this.currentY += height
        return group
    }
}
```

---

## 6. State Machine Diagram

### Features

#### 6.1 States
- **Rounded rectangles** for states
- **Entry/Exit actions**
- **Internal transitions**
- **Composite states** (states within states)
- **Initial/Final states** (special symbols)

#### 6.2 Transitions
- **Arrows between states**
- **Transition labels**: event [guard] / action
- **Curved paths** for readability

### Syntax

```arc
component "Brake Controller" {
    id: "LC-001"
    
    state_machine "Brake Control FSM" {
        initial_state: Idle
        
        // Simple state
        state Idle {
            entry_action: "Initialize sensors"
            exit_action: "Log state change"
            color: "#D0E8FF"
        }
        
        // Composite state (contains sub-states)
        state Active {
            color: "#FFE699"
            
            // Sub-states
            initial_state: Monitoring
            
            state Monitoring {
                entry_action: "Start monitoring"
                internal_transition: "SensorUpdate / UpdateDisplay"
            }
            
            state Braking {
                entry_action: "Apply brakes"
                exit_action: "Release brakes"
            }
        }
        
        state Error {
            color: "#FF9999"
            entry_action: "Trigger alarm"
        }
        
        final_state: Shutdown
        
        // Transitions
        transition {
            from: Idle
            to: Active
            trigger: "StartCommand"
            guard: "system_ready == true"
            action: "Enable monitoring"
        }
        
        transition {
            from: "Active.Monitoring"
            to: "Active.Braking"
            trigger: "ObstacleDetected"
            guard: "distance < threshold"
            action: "CalculateBrakeForce()"
            timing: "< 5ms"
        }
        
        transition {
            from: "Active.Braking"
            to: "Active.Monitoring"
            trigger: "ObstacleClear"
        }
        
        transition {
            from: Active
            to: Error
            trigger: "FaultDetected"
            priority: high
        }
        
        transition {
            from: Error
            to: Idle
            trigger: "Reset"
        }
        
        transition {
            from: Idle
            to: Shutdown
            trigger: "PowerOff"
        }
    }
}
```

---

## 7. Custom Layout Engines

### 7.1 Swimlane Layout (for OAD)
```typescript
class SwimlaneLayoutEngine {
    layout(diagram: OperationalDiagram): LayoutResult {
        // 1. Group activities by entity (horizontal lanes)
        // 2. Position activities to minimize crossings
        // 3. Route data flows with minimal bends
        // 4. Adjust lane heights based on content
    }
}
```

### 7.2 Timeline Layout (for Sequence Diagrams)
```typescript
class TimelineLayoutEngine {
    layout(scenario: Scenario): LayoutResult {
        // 1. Position participants horizontally
        // 2. Order messages vertically by time
        // 3. Calculate activation box positions
        // 4. Layout combined fragments
    }
}
```

### 7.3 Hierarchical Layout (for Capabilities)
```typescript
class HierarchicalLayoutEngine {
    layout(capabilities: Capability[]): LayoutResult {
        // 1. Build tree structure
        // 2. Calculate box nesting
        // 3. Apply Reingold-Tilford algorithm
        // 4. Adjust for labels and relationships
    }
}
```

---

## Implementation Phases

### Phase 1: Foundation (4 weeks)
1. **Week 1-2**: Extend ArcLang syntax for all diagram types
2. **Week 3**: Implement basic renderer infrastructure
3. **Week 4**: Create SVG helper library

### Phase 2: Operational Diagrams (6 weeks)
4. **Week 5-6**: Operational Activity Diagrams (swimlanes, activities)
5. **Week 7-8**: Capability Decomposition
6. **Week 9-10**: Actor relationships and data flows

### Phase 3: Functional Diagrams (5 weeks)
7. **Week 11-12**: Functional architecture with ports
8. **Week 13-14**: Data flow rendering
9. **Week 15**: Function hierarchy

### Phase 4: Physical Diagrams (4 weeks)
10. **Week 16-17**: Physical nodes and behavior components
11. **Week 18**: Allocation visualization
12. **Week 19**: Physical links

### Phase 5: Sequence Diagrams (6 weeks)
13. **Week 20-21**: Lifelines and basic messages
14. **Week 22-23**: Combined fragments (PAR, OPT, LOOP, ALT)
15. **Week 24**: Activation boxes
16. **Week 25**: Timing constraints

### Phase 6: State Machines (4 weeks)
17. **Week 26-27**: States and transitions
18. **Week 28**: Composite states
19. **Week 29**: Actions and guards

### Phase 7: Polish & Performance (3 weeks)
20. **Week 30**: Export to PNG/SVG/PDF
21. **Week 31**: Interactive features (hover, click, pan/zoom)
22. **Week 32**: Performance optimization

---

## Technology Stack

### Backend (Rust)
- **Parser**: Extend for new syntax
- **Semantic model**: Rich diagram data structures
- **JSON export**: Complete diagram specifications

### Diagram Service (Node.js/TypeScript)
```
apps/
├── diagram-service/
│   ├── src/
│   │   ├── renderers/
│   │   │   ├── operational-activity-renderer.ts
│   │   │   ├── capability-renderer.ts
│   │   │   ├── functional-renderer.ts
│   │   │   ├── physical-renderer.ts
│   │   │   ├── sequence-renderer.ts
│   │   │   └── state-machine-renderer.ts
│   │   ├── layouts/
│   │   │   ├── swimlane-layout.ts
│   │   │   ├── timeline-layout.ts
│   │   │   ├── hierarchical-layout.ts
│   │   │   └── force-directed-layout.ts
│   │   ├── svg/
│   │   │   ├── primitives.ts
│   │   │   ├── shapes.ts
│   │   │   └── helpers.ts
│   │   └── index.ts
│   └── package.json
```

### Frontend (React/TypeScript)
```
apps/web/components/
├── diagrams/
│   ├── operational-activity-diagram.tsx
│   ├── capability-diagram.tsx
│   ├── functional-diagram.tsx
│   ├── physical-diagram.tsx
│   ├── sequence-diagram.tsx
│   └── state-machine-diagram.tsx
```

---

## Success Metrics

- ✅ **Diagram Types**: 6 (OAD, Capability, Functional, Physical, Sequence, State)
- ✅ **Visual Fidelity**: Match Capella quality
- ✅ **Rendering Speed**: < 500ms for typical diagrams
- ✅ **Export Formats**: PNG, SVG, PDF
- ✅ **Interactive**: Pan, zoom, hover tooltips
- ✅ **Browser Support**: Chrome, Firefox, Safari, Edge

---

## Next Steps

1. **Review and approve** this comprehensive plan
2. **Set up diagram-service** project structure
3. **Begin Phase 1** (syntax extensions)
4. **Create proof-of-concept** for OAD swimlanes
5. **Iterate** based on feedback

---

**This is a 32-week project (8 months) to achieve complete Capella-quality diagram capabilities.** 🚀

Ready to start implementation?
