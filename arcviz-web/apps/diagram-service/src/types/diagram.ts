/**
 * Diagram Rendering Types
 * 
 * Common types for diagram layout and rendering
 */

// ============================================================================
// Layout Types
// ============================================================================

export interface Point {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Rect extends Point, Size {}

export interface Padding {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

// ============================================================================
// SVG Elements
// ============================================================================

export interface SvgElement {
  type: 'rect' | 'circle' | 'path' | 'text' | 'line' | 'polyline' | 'polygon' | 'g' | 'marker' | 'defs' | 'filter' | 'feGaussianBlur' | 'feOffset' | 'feComponentTransfer' | 'feFuncA' | 'feMerge' | 'feMergeNode';
  attributes: Record<string, string | number>;
  children?: SvgElement[];
  text?: string;
}

// ============================================================================
// Node Types
// ============================================================================

export interface DiagramNode {
  id: string;
  label: string;
  type: NodeType;
  position?: Point;
  size?: Size;
  color?: string;
  icon?: string;
  children?: DiagramNode[];
  ports?: Port[];
  metadata?: Record<string, any>;
}

export type NodeType =
  | 'activity'           // Operational activities (yellow boxes)
  | 'actor'              // Actors (stick figures)
  | 'function'           // System functions (green boxes)
  | 'component'          // Logical components (blue boxes)
  | 'physical-node'      // Physical nodes (yellow boxes)
  | 'behavior'           // Behavior components (blue)
  | 'hardware'           // Hardware components (gray)
  | 'capability'         // Capabilities
  | 'state'              // State machine states (regular)
  | 'initial-state'      // Initial state (filled circle)
  | 'final-state'        // Final state (double circle)
  | 'composite-state'    // Composite state (contains sub-states)
  | 'choice'             // Choice pseudo-state (diamond)
  | 'junction'           // Junction pseudo-state (diamond)
  | 'lifeline'           // Sequence diagram lifelines
  | 'class'              // Class (UML class)
  | 'interface'          // Interface (UML interface)
  | 'datastructure';     // Data structure (bit-precise)

export interface Port {
  id: string;
  name: string;
  direction: 'IN' | 'OUT' | 'INOUT';
  side: 'TOP' | 'RIGHT' | 'BOTTOM' | 'LEFT';
  position?: Point;
  metadata?: Record<string, any>;
}

// ============================================================================
// Edge Types
// ============================================================================

export interface DiagramEdge {
  id: string;
  from: string;
  to: string;
  fromPort?: string;
  toPort?: string;
  label?: string;
  type: EdgeType;
  points?: Point[];
  color?: string;
  metadata?: Record<string, any>;
}

export type EdgeType =
  | 'operational-exchange'    // Data flows in OA
  | 'functional-exchange'     // Port-to-port flows in SA
  | 'component-exchange'      // Component-to-component flows in LA
  | 'physical-link'           // Physical connections in PA
  | 'transition'              // State machine transitions
  | 'message-sync'            // Synchronous messages
  | 'message-async'           // Asynchronous messages
  | 'message-return'          // Return messages
  | 'association'             // General associations
  | 'composition'             // Composition (filled diamond)
  | 'aggregation'             // Aggregation (hollow diamond)
  | 'generalization'          // Generalization/inheritance
  | 'allocation'              // Allocation links
  | 'hierarchy';              // Tree hierarchy edges

// ============================================================================
// Diagram Types
// ============================================================================

export interface Diagram {
  title: string;
  type: DiagramType;
  nodes: DiagramNode[];
  edges: DiagramEdge[];
  swimlanes?: Swimlane[];
  fragments?: DiagramFragment[];
  metadata?: Record<string, any>;
}

export type DiagramType =
  | 'operational-activity'
  | 'capability-decomposition'
  | 'functional-dataflow'
  | 'component-block'
  | 'physical-deployment'
  | 'sequence'
  | 'state-machine'
  | 'data-model';

export interface Swimlane {
  id: string;
  label: string;
  activityIds: string[];
  position?: Point;
  size?: Size;
}

export interface DiagramFragment {
  id: string;
  type: 'PAR' | 'OPT' | 'LOOP' | 'ALT';
  label: string;
  condition?: string;
  operands: DiagramFragmentOperand[];
  position?: Point;
  size?: Size;
}

export interface DiagramFragmentOperand {
  label: string;
  messageIds: string[];
}

// ============================================================================
// Layout Configuration
// ============================================================================

export interface LayoutConfig {
  direction?: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT';
  nodeSpacing?: number;
  layerSpacing?: number;
  edgeSpacing?: number;
  padding?: Padding;
  algorithm?: 'elk' | 'dagre' | 'swimlane' | 'timeline';
}

export interface SwimlaneLayoutConfig extends LayoutConfig {
  swimlaneWidth?: number;
  swimlaneSpacing?: number;
  activityHeight?: number;
  activityWidth?: number;
}

export interface TimelineLayoutConfig extends LayoutConfig {
  lifelineSpacing?: number;
  messageSpacing?: number;
  fragmentPadding?: Padding;
  activationWidth?: number;
}

// ============================================================================
// Rendering Configuration
// ============================================================================

export interface RenderConfig {
  width?: number;
  height?: number;
  backgroundColor?: string;
  fontSize?: number;
  fontFamily?: string;
  showGrid?: boolean;
  showLabels?: boolean;
  colorScheme?: ColorScheme;
}

export interface ColorScheme {
  activity: string;          // Operational activities (#FFD966)
  actor: string;              // Actors (#2E75B6)
  function: string;           // System functions (#70AD47)
  component: string;          // Logical components (#5B9BD5)
  physicalNode: string;       // Physical nodes (#FFE699)
  behavior: string;           // Behavior components (#5B9BD5)
  hardware: string;           // Hardware components (#C0C0C0)
  capability: string;         // Capabilities (#FFC000)
  state: string;              // States (#BDD7EE)
  edge: string;               // Default edge color (#000000)
  text: string;               // Text color (#000000)
  background: string;         // Background (#FFFFFF)
}

// Default Capella color scheme
export const CAPELLA_COLORS: ColorScheme = {
  activity: '#FFB266',       // CORRECTED: Operational Activity (was #FFD966)
  actor: '#FFFF99',          // CORRECTED: Operational Actor/Entity (was #2E75B6)
  function: '#ADD8E6',       // CORRECTED: System Function (was #70AD47)
  component: '#6495ED',      // CORRECTED: Logical Component (was #5B9BD5)
  physicalNode: '#FFD700',   // CORRECTED: Physical Node/Hardware (was #FFE699)
  behavior: '#4169E1',       // CORRECTED: Physical Behavioral/Software (was #5B9BD5)
  hardware: '#FFD700',       // Physical Node (same as physicalNode)
  capability: '#FFC000',     // Capability (unchanged)
  state: '#BDD7EE',          // State (unchanged)
  edge: '#808080',           // CORRECTED: Interactions/Exchanges (was #000000)
  text: '#000000',           // Text (unchanged)
  background: '#FFFFFF',     // Background (unchanged)
};

// ============================================================================
// Export Types
// ============================================================================

export interface ExportOptions {
  format: 'svg' | 'png' | 'pdf';
  scale?: number;
  quality?: number;
}

export interface DiagramOutput {
  svg: string;
  width: number;
  height: number;
  metadata?: Record<string, any>;
}
