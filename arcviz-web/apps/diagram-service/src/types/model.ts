/**
 * ArcLang Model Types
 * 
 * TypeScript types matching the JSON export from ArcLang compiler.
 * These types enable type-safe diagram rendering for all Capella diagram types.
 */

// ============================================================================
// Attribute Values
// ============================================================================

export type AttributeValue =
  | { String: string }
  | { Number: number }
  | { Boolean: boolean }
  | { List: AttributeValue[] };

export type Attributes = Record<string, AttributeValue>;

// ============================================================================
// Top-Level Model
// ============================================================================

export interface Model {
  operational_analysis: OperationalAnalysis[];
  system_analysis: SystemAnalysis[];
  logical_architecture: LogicalArchitecture[];
  physical_architecture: PhysicalArchitecture[];
  epbs: Epbs[];
  safety_analysis: SafetyAnalysis[];
  traces: Trace[];
  state_machines: StateMachine[];
  scenarios: Scenario[];
  exchange_items: ExchangeItem[];
  data_types: DataType[];
}

// ============================================================================
// Operational Analysis (OA) - Swimlane Diagrams
// ============================================================================

export interface OperationalAnalysis {
  name: string;
  actors: Actor[];
  entities: Entity[];
  capabilities: OperationalCapability[];
  processes: OperationalProcess[];
  roles: OperationalRole[];
  entity_capability_involvements: EntityOperationalCapabilityInvolvement[];
  activities: OperationalActivity[];
  exchanges: OperationalExchange[];
  capability_associations: CapabilityAssociation[];
}

export interface Actor {
  name: string;
  id: string | null;
  icon: string;
  attributes: Attributes;
}

export interface Entity {
  id: string;
  name: string;
  entity_type: 'Actor' | 'System' | 'Environment';
  icon: string;
  description: string | null;
  attributes: Attributes;
}

export interface OperationalCapability {
  id: string;
  name: string;
  level: 'Mission' | 'Capability' | 'SubCapability';
  color: string | null;
  stereotype: string | null;
  children: OperationalCapability[];
  involved_entities: string[];
  operational_processes: string[];
  attributes: Attributes;
}

export interface OperationalProcess {
  id: string;
  name: string;
  description: string | null;
  activities: string[];
  pre_condition: string | null;
  post_condition: string | null;
  attributes: Attributes;
}

export interface OperationalRole {
  id: string;
  name: string;
  description: string | null;
  entity_id: string;
  allocated_activities: string[];
  attributes: Attributes;
}

export interface EntityOperationalCapabilityInvolvement {
  entity_id: string;
  capability_id: string;
  involvement_kind: 'responsible' | 'accountable' | 'consulted' | 'informed';
  attributes: Attributes;
}

export interface OperationalActivity {
  id: string;
  name: string;
  performed_by: string;
  category: string;
  icon: string;
  color: string;
  sub_activities: OperationalActivity[];
  attributes: Attributes;
}

export interface OperationalExchange {
  from: string;
  to: string;
  data_type: string;
  label: string | null;
  protocol: string | null;
  attributes: Attributes;
}

export interface CapabilityAssociation {
  from: string;
  to: string;
  association_type: string;
  label: string | null;
}

// ============================================================================
// System Analysis (SA) - Functional Dataflow Diagrams
// ============================================================================

export interface SystemAnalysis {
  name: string;
  requirements: Requirement[];
  missions: Mission[];
  functions: SystemFunction[];
  components: SystemComponent[];
  external_actors: ExternalActor[];
  functional_exchanges: FunctionalExchange[];
  capability_realizations: CapabilityRealization[];
}

export interface Mission {
  id: string;
  name: string;
  description: string | null;
  capabilities: string[];
  operational_activities: string[];
  attributes: Attributes;
}

export interface CapabilityRealization {
  id: string;
  name: string;
  description: string | null;
  realized_capability_id: string;
  involved_components: string[];
  involved_functions: string[];
  scenarios: string[];
  attributes: Attributes;
}

export interface Requirement {
  id: string;
  name: string;
  description: string | null;
  requirement_type: 'Functional' | 'NonFunctional' | 'Performance' | 'Safety' | 'Security';
  priority: 'Critical' | 'High' | 'Medium' | 'Low';
  verification_method: 'Test' | 'Analysis' | 'Inspection' | 'Demonstration';
  derived_from: string[];
  satisfied_by: string[];
  verified_by: string[];
  attributes: Attributes;
}

export interface SystemFunction {
  id: string;
  name: string;
  category: 'Environmental' | 'System' | 'Management' | 'Control' | 'Interaction';
  color: string | null;
  icon: string | null;
  ports: FunctionPort[];
  sub_functions: SystemFunction[];
  attributes: Attributes;
}

export interface FunctionPort {
  name: string;
  direction: 'In' | 'Out' | 'InOut';
  port_type: 'Data' | 'Control' | 'Event';
  data_type: string;
}

export interface ExternalActor {
  id: string;
  name: string;
  color: string;
  attributes: Attributes;
}

export interface FunctionalExchange {
  from_port: string;
  to_port: string;
  data_type: string;
  label: string | null;
}

export interface SystemComponent {
  name: string;
  attributes: Attributes;
}

// ============================================================================
// Logical Architecture (LA) - Component Block Diagrams
// ============================================================================

export interface LogicalArchitecture {
  name: string;
  components: LogicalComponent[];
  interfaces: LogicalInterface[];
  component_exchanges: ComponentExchange[];
  unallocated_functions: string[];
}

export interface LogicalComponent {
  id: string;
  name: string;
  component_type: string;
  color: string | null;
  sub_components: LogicalComponent[];
  allocated_functions: string[];
  ports: ComponentPort[];
  functions: LogicalFunction[];
  interfaces_in: InterfaceDefinition[];
  interfaces_out: InterfaceDefinition[];
  attributes: Attributes;
}

export interface ComponentPort {
  name: string;
  direction: 'In' | 'Out' | 'InOut';
  interface_type: string;
}

export interface ComponentExchange {
  from_port: string;
  to_port: string;
  exchange_item: string;
  label: string | null;
}

export interface InterfaceDefinition {
  name: string;
  protocol: string | null;
  format: string | null;
  attributes: Attributes;
}

export interface LogicalFunction {
  name: string;
  attributes: Attributes;
}

export interface LogicalInterface {
  name: string;
  from: string;
  to: string;
  attributes: Attributes;
}

// ============================================================================
// Physical Architecture (PA) - Deployment Diagrams
// ============================================================================

export interface PhysicalArchitecture {
  name: string;
  nodes: PhysicalNode[];
  links: PhysicalLink[];
  physical_exchanges: PhysicalExchange[];
  physical_paths: PhysicalPath[];
  deployment_links: DeploymentLink[];
}

export interface PhysicalPath {
  id: string;
  name: string;
  description: string | null;
  start_node: string;
  end_node: string;
  intermediate_links: string[];
  latency: number | null;
  bandwidth: string | null;
  allocated_exchanges: string[];
  attributes: Attributes;
}

export interface DeploymentLink {
  id: string;
  deployed_element_id: string;
  target_node_id: string;
  deployment_type: 'allocates' | 'deploys' | 'implements';
  attributes: Attributes;
}

export interface PhysicalNode {
  id: string;
  name: string;
  node_type: 'Hardware' | 'Software' | 'SystemOfSystems';
  color: string | null;
  processor: string | null;
  memory: string | null;
  behavior_components: BehaviorComponent[];
  hardware_components: HardwareComponent[];
  deployments: Deployment[];
  attributes: Attributes;
}

export interface BehaviorComponent {
  id: string;
  name: string;
  allocated_functions: string[];
  color: string | null;
}

export interface HardwareComponent {
  id: string;
  name: string;
  hw_type: string;
  specs: string | null;
  color: string | null;
}

export interface Deployment {
  component: string;
  attributes: Attributes;
}

export interface PhysicalLink {
  from: string;
  to: string;
  protocol: string;
  bandwidth: string | null;
  color: string | null;
  connections: string[];
  attributes: Attributes;
}

export interface PhysicalExchange {
  from: string;
  to: string;
  via: string | null;
  message_type: string;
  frequency: string | null;
  label: string | null;
}

// ============================================================================
// Behavioral Models - Sequence Diagrams
// ============================================================================

export interface Scenario {
  name: string;
  participants: Participant[];
  messages: Message[];
  fragments: CombinedFragment[];
  timing_constraints: TimingConstraint[];
}

export interface Participant {
  id: string;
  name: string;
  participant_type: 'Actor' | 'Component' | 'System';
  lifeline_color: string;
}

export interface Message {
  from: string;
  to: string;
  label: string;
  message_type: 'Synchronous' | 'Asynchronous' | 'Return';
  activation: boolean;
  timing: string | null;
  params: string | null;
}

export interface CombinedFragment {
  fragment_type: 'Par' | 'Opt' | 'Loop' | 'Alt';
  label: string;
  condition: string | null;
  operands: FragmentOperand[];
}

export interface FragmentOperand {
  label: string;
  messages: Message[];
}

export interface TimingConstraint {
  from_message: string;
  to_message: string;
  max_duration: string;
  requirement: string | null;
}

// ============================================================================
// Behavioral Models - State Machines
// ============================================================================

export interface StateMachine {
  name: string;
  initial_state: string;
  states: State[];
  transitions: Transition[];
  modes: Mode[];
  regions: Region[];
}

export interface State {
  name: string;
  state_type: 'Initial' | 'Final' | 'Simple' | 'Composite' | 'Choice' | 'Junction';
  entry_actions: string[];
  do_activities: string[];
  exit_actions: string[];
  internal_transitions: InternalTransition[];
  sub_states: State[];
  color: string | null;
  attributes: Attributes;
}

export interface Mode {
  id: string;
  name: string;
  description: string | null;
  is_initial: boolean;
  states: string[];
  mode_transitions: ModeTransition[];
  attributes: Attributes;
}

export interface ModeTransition {
  from_mode: string;
  to_mode: string;
  trigger: string;
  guard: Guard | null;
  attributes: Attributes;
}

export interface Region {
  id: string;
  name: string;
  states: string[];
  attributes: Attributes;
}

export interface Transition {
  id: string;
  from: string;
  to: string;
  trigger: string;
  guard: Guard | null;
  effect: string | null;
  timing: string | null;
  priority: number | null;
  kind: 'internal' | 'local' | 'external';
  attributes: Attributes;
}

export interface Guard {
  id: string;
  expression: string;
  description: string | null;
  language: 'OCL' | 'Natural' | 'Expression';
  attributes: Attributes;
}

export interface InternalTransition {
  trigger: string;
  guard: Guard | null;
  effect: string | null;
}

// ============================================================================
// Data Models
// ============================================================================

export interface ExchangeItem {
  name: string;
  stereotype: string;
  attributes: DataAttribute[];
}

export interface DataAttribute {
  name: string;
  attr_type: string;
  default_value: string | null;
  enumeration: string[] | null;
}

export interface DataType {
  id: string;
  name: string;
  description: string | null;
  type_kind: 'Primitive' | 'Composite' | 'Collection' | 'Enumeration' | 'Union';
  base_type: string | null;
  min_value: number | null;
  max_value: number | null;
  unit: string | null;
  pattern: string | null;
  bit_size: number | null;
  fields: DataField[];
  enumeration_values: EnumValue[];
  constraints: Constraint[];
  attributes: Attributes;
}

export interface DataField {
  name: string;
  type: string;
  multiplicity: string;
  default_value: string | null;
  description: string | null;
}

export interface EnumValue {
  name: string;
  value: string | null;
  description: string | null;
}

export interface Constraint {
  id: string;
  name: string;
  expression: string;
  language: 'OCL' | 'Natural' | 'Expression';
  constrained_elements: string[];
  description: string | null;
  attributes: Attributes;
}

// ============================================================================
// Product Breakdown Structure (EPBS)
// ============================================================================

export interface Epbs {
  name: string;
  systems: EpbsSystem[];
}

export interface EpbsSystem {
  name: string;
  subsystems: EpbsSubsystem[];
  attributes: Attributes;
}

export interface EpbsSubsystem {
  name: string;
  items: EpbsItem[];
  attributes: Attributes;
}

export interface EpbsItem {
  name: string;
  attributes: Attributes;
}

// ============================================================================
// Safety Analysis
// ============================================================================

export interface SafetyAnalysis {
  hazards: Hazard[];
  fmea: FmeaEntry[];
  attributes: Attributes;
}

export interface Hazard {
  name: string;
  attributes: Attributes;
}

export interface FmeaEntry {
  name: string;
  attributes: Attributes;
}

// ============================================================================
// Traceability
// ============================================================================

export interface Trace {
  from: string;
  to: string;
  trace_type: string;
  attributes: Attributes;
}

// ============================================================================
// Class Diagrams
// ============================================================================

export interface ClassModel {
  name: string;
  classes?: Class[];
  interfaces?: Interface[];
  data_structures?: DataStructure[];
  associations?: Association[];
  generalizations?: Generalization[];
}

export interface Class {
  name: string;
  stereotype?: string;
  is_abstract?: boolean;
  attributes?: ClassAttribute[];
  operations?: Operation[];
}

export interface Interface {
  name: string;
  operations?: Operation[];
}

export interface DataStructure {
  name: string;
  bit_size?: number;
  fields?: Field[];
}

export interface ClassAttribute {
  name: string;
  type: string;
  visibility?: 'public' | 'private' | 'protected' | 'package';
  default_value?: string;
}

export interface Operation {
  name: string;
  visibility?: 'public' | 'private' | 'protected' | 'package';
  return_type?: string;
  parameters?: Parameter[];
  is_abstract?: boolean;
}

export interface Parameter {
  name: string;
  type: string;
}

export interface Field {
  name: string;
  type: string;
  bit_size?: number;
}

export interface Association {
  from: string;
  to: string;
  type: 'association' | 'composition' | 'aggregation';
  label?: string;
  multiplicity_from?: string;
  multiplicity_to?: string;
}

export interface Generalization {
  child: string;
  parent: string;
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Extract string value from AttributeValue
 */
export function getStringAttribute(attrs: Attributes, key: string): string | null {
  const value = attrs[key];
  if (value && 'String' in value) {
    return value.String;
  }
  return null;
}

/**
 * Extract number value from AttributeValue
 */
export function getNumberAttribute(attrs: Attributes, key: string): number | null {
  const value = attrs[key];
  if (value && 'Number' in value) {
    return value.Number;
  }
  return null;
}

/**
 * Extract boolean value from AttributeValue
 */
export function getBooleanAttribute(attrs: Attributes, key: string): boolean | null {
  const value = attrs[key];
  if (value && 'Boolean' in value) {
    return value.Boolean;
  }
  return null;
}

/**
 * Extract list value from AttributeValue
 */
export function getListAttribute(attrs: Attributes, key: string): AttributeValue[] | null {
  const value = attrs[key];
  if (value && 'List' in value) {
    return value.List;
  }
  return null;
}
