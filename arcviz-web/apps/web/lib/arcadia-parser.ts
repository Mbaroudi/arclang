// Parser ArcLang avec support des 7 dimensions Arcadia

export interface ArcadiaModel {
  operational: OperationalAnalysis;
  system: SystemAnalysis;
  logical: LogicalArchitecture;
  physical: PhysicalArchitecture;
  epbs: EPBSStructure;
  requirements: RequirementsModel;
  crossCutting: CrossCuttingConcerns;
}

// === 1. OPERATIONAL ANALYSIS ===
export interface OperationalAnalysis {
  actors: OperationalActor[];
  activities: OperationalActivity[];
  entities: OperationalEntity[];
  interactions: OperationalInteraction[];
}

export interface OperationalActor {
  id: string;
  name: string;
  type: 'human' | 'external_system' | 'environment';
  goals?: string[];
  capabilities?: string[];
}

export interface OperationalActivity {
  id: string;
  name: string;
  description?: string;
  inputs?: string[];
  outputs?: string[];
  actors?: string[];
}

export interface OperationalEntity {
  id: string;
  name: string;
  type: string;
  properties?: Record<string, any>;
}

export interface OperationalInteraction {
  id: string;
  from: string;
  to: string;
  type: 'exchange' | 'control' | 'trigger';
  data?: string;
}

// === 2. SYSTEM ANALYSIS ===
export interface SystemAnalysis {
  functions: SystemFunction[];
  actors: SystemActor[];
  capabilities: SystemCapability[];
  context: SystemContext;
}

export interface SystemFunction {
  id: string;
  name: string;
  description?: string;
  inputs?: string[];
  outputs?: string[];
  safetyLevel?: string;
  traces?: string[];
}

export interface SystemActor {
  id: string;
  name: string;
  type: 'user' | 'external_system' | 'device';
  interfaces?: string[];
}

export interface SystemCapability {
  id: string;
  name: string;
  description?: string;
  conditions?: string[];
  involves?: string[];
}

export interface SystemContext {
  system: string;
  actors: string[];
  interfaces: SystemInterface[];
}

export interface SystemInterface {
  id: string;
  from: string;
  to: string;
  protocol?: string;
  data?: string[];
}

// === 3. LOGICAL ARCHITECTURE ===
export interface LogicalArchitecture {
  components: LogicalComponent[];
  functions: LogicalFunction[];
  interfaces: LogicalInterface[];
  functionalChains: FunctionalChain[];
}

export interface LogicalComponent {
  id: string;
  name: string;
  type: 'logical' | 'abstract';
  functions?: string[];
  subComponents?: string[];
  allocatedTo?: string[];
}

export interface LogicalFunction {
  id: string;
  name: string;
  component?: string;
  inputs?: string[];
  outputs?: string[];
  algorithm?: string;
}

export interface LogicalInterface {
  id: string;
  from: string;
  to: string;
  type: 'data' | 'control' | 'service';
  protocol?: string;
}

export interface FunctionalChain {
  id: string;
  name: string;
  functions: string[];
  dataFlow?: string[];
}

// === 4. PHYSICAL ARCHITECTURE ===
export interface PhysicalArchitecture {
  nodes: PhysicalNode[];
  components: PhysicalComponent[];
  links: PhysicalLink[];
  deployment: DeploymentAllocation[];
}

export interface PhysicalNode {
  id: string;
  name: string;
  type: 'computer' | 'sensor' | 'actuator' | 'network';
  hardware?: string;
  os?: string;
  memory?: string;
  cpu?: string;
}

export interface PhysicalComponent {
  id: string;
  name: string;
  deployedOn?: string;
  logicalComponent?: string;
  technology?: string;
}

export interface PhysicalLink {
  id: string;
  from: string;
  to: string;
  protocol: string;
  bandwidth?: string;
  latency?: string;
}

export interface DeploymentAllocation {
  logicalComponent: string;
  physicalNode: string;
  constraints?: string[];
}

// === 5. EPBS (End Product Breakdown Structure) ===
export interface EPBSStructure {
  products: Product[];
  configurationItems: ConfigurationItem[];
  breakdown: ProductHierarchy[];
}

export interface Product {
  id: string;
  name: string;
  version: string;
  type: 'system' | 'subsystem' | 'component';
  children?: string[];
}

export interface ConfigurationItem {
  id: string;
  name: string;
  type: 'hardware' | 'software' | 'integration' | 'documentation';
  version: string;
  status: 'planned' | 'in_progress' | 'completed' | 'delivered';
}

export interface ProductHierarchy {
  parent: string;
  children: string[];
  integrationType?: string;
}

// === 6. REQUIREMENTS ===
export interface RequirementsModel {
  requirements: Requirement[];
  traces: TraceLink[];
  verifications: Verification[];
}

export interface Requirement {
  id: string;
  name: string;
  text: string;
  type: 'functional' | 'non_functional' | 'constraint' | 'performance' | 'safety';
  priority?: 'critical' | 'high' | 'medium' | 'low';
  safetyLevel?: string;
  status?: 'draft' | 'approved' | 'implemented' | 'verified';
  allocatedTo?: string[];
}

export interface TraceLink {
  from: string;
  to: string;
  type: 'satisfies' | 'derives' | 'refines' | 'verifies';
}

export interface Verification {
  id: string;
  requirementId: string;
  method: 'test' | 'analysis' | 'inspection' | 'demonstration';
  status: 'planned' | 'passed' | 'failed';
  details?: string;
}

// === 7. CROSS-CUTTING CONCERNS ===
export interface CrossCuttingConcerns {
  safety: SafetyAnalysis;
  modesStates: ModesStates;
  sequences: Sequence[];
  interfaces: SystemInterface[];
}

export interface SafetyAnalysis {
  hazards: Hazard[];
  fmea: FMEA[];
  fta: FTA[];
}

export interface Hazard {
  id: string;
  name: string;
  description: string;
  severity: number;
  probability: number;
  risk: number;
  mitigations?: string[];
}

export interface FMEA {
  id: string;
  component: string;
  failureMode: string;
  effect: string;
  severity: number;
  occurrence: number;
  detection: number;
  rpn: number;
  actions?: string[];
}

export interface FTA {
  id: string;
  topEvent: string;
  gates: FTAGate[];
  basicEvents: string[];
}

export interface FTAGate {
  id: string;
  type: 'AND' | 'OR' | 'XOR';
  inputs: string[];
  output: string;
}

export interface ModesStates {
  modes: Mode[];
  states: State[];
  transitions: Transition[];
}

export interface Mode {
  id: string;
  name: string;
  description?: string;
  entryConditions?: string[];
  exitConditions?: string[];
}

export interface State {
  id: string;
  name: string;
  mode?: string;
  type: 'initial' | 'normal' | 'degraded' | 'final';
}

export interface Transition {
  id: string;
  from: string;
  to: string;
  trigger: string;
  guard?: string;
  action?: string;
}

export interface Sequence {
  id: string;
  name: string;
  participants: string[];
  steps: SequenceStep[];
}

export interface SequenceStep {
  id: string;
  from: string;
  to: string;
  message: string;
  type: 'sync' | 'async' | 'return';
  order: number;
}

// === PARSER ===
export class ArcadiaParser {
  private code: string;

  constructor(code: string) {
    this.code = code;
  }

  parse(): ArcadiaModel {
    return {
      operational: this.parseOperational(),
      system: this.parseSystem(),
      logical: this.parseLogical(),
      physical: this.parsePhysical(),
      epbs: this.parseEPBS(),
      requirements: this.parseRequirements(),
      crossCutting: this.parseCrossCutting(),
    };
  }

  private parseOperational(): OperationalAnalysis {
    const actors: OperationalActor[] = [];
    const activities: OperationalActivity[] = [];
    const entities: OperationalEntity[] = [];
    const interactions: OperationalInteraction[] = [];

    // Parse operational_analysis blocks
    const opRegex = /operational_analysis\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = opRegex.exec(this.code)) !== null) {
      const body = match[2];
      
      // Parse actors
      const actorRegex = /actor\s+"([^"]+)"\s*{([^}]+)}/g;
      let actorMatch;
      while ((actorMatch = actorRegex.exec(body)) !== null) {
        actors.push({
          id: `OA-${actors.length + 1}`,
          name: actorMatch[1],
          type: 'human',
          goals: this.extractArray(actorMatch[2], 'goal'),
          capabilities: this.extractArray(actorMatch[2], 'capability'),
        });
      }

      // Parse operational activities
      const activityRegex = /operational_activity\s+"([^"]+)"\s*{([^}]+)}/g;
      let actMatch;
      while ((actMatch = activityRegex.exec(body)) !== null) {
        activities.push({
          id: `OA-ACT-${activities.length + 1}`,
          name: actMatch[1],
          inputs: this.extractArray(actMatch[2], 'input'),
          outputs: this.extractArray(actMatch[2], 'output'),
        });
      }
    }

    return { actors, activities, entities, interactions };
  }

  private parseSystem(): SystemAnalysis {
    const functions: SystemFunction[] = [];
    const actors: SystemActor[] = [];
    const capabilities: SystemCapability[] = [];

    // Parse system functions
    const funcRegex = /system_function\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = funcRegex.exec(this.code)) !== null) {
      functions.push({
        id: `SF-${functions.length + 1}`,
        name: match[1],
        inputs: this.extractArray(match[2], 'input'),
        outputs: this.extractArray(match[2], 'output'),
        safetyLevel: this.extractValue(match[2], 'safety_level'),
      });
    }

    // Parse system capabilities
    const capRegex = /system_capability\s+"([^"]+)"\s*{([^}]+)}/g;
    while ((match = capRegex.exec(this.code)) !== null) {
      capabilities.push({
        id: `SC-${capabilities.length + 1}`,
        name: match[1],
        description: this.extractValue(match[2], 'description'),
      });
    }

    return {
      functions,
      actors,
      capabilities,
      context: { system: 'Main System', actors: [], interfaces: [] },
    };
  }

  private parseLogical(): LogicalArchitecture {
    const components: LogicalComponent[] = [];
    const functions: LogicalFunction[] = [];
    const interfaces: LogicalInterface[] = [];
    const functionalChains: FunctionalChain[] = [];

    // Parse components
    const compRegex = /component\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = compRegex.exec(this.code)) !== null) {
      const body = match[2];
      const componentId = `LC-${components.length + 1}`;
      
      // Extract functions from component
      const funcNames: string[] = [];
      const funcRegex = /function\s+"([^"]+)"/g;
      let funcMatch;
      while ((funcMatch = funcRegex.exec(body)) !== null) {
        const funcId = `LF-${functions.length + 1}`;
        funcNames.push(funcId);
        functions.push({
          id: funcId,
          name: funcMatch[1],
          component: componentId,
        });
      }

      components.push({
        id: componentId,
        name: match[1],
        type: 'logical',
        functions: funcNames,
      });
    }

    return { components, functions, interfaces, functionalChains };
  }

  private parsePhysical(): PhysicalArchitecture {
    const nodes: PhysicalNode[] = [];
    const components: PhysicalComponent[] = [];
    const links: PhysicalLink[] = [];
    const deployment: DeploymentAllocation[] = [];

    // Parse physical nodes
    const nodeRegex = /node\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = nodeRegex.exec(this.code)) !== null) {
      nodes.push({
        id: `PN-${nodes.length + 1}`,
        name: match[1],
        type: 'computer',
        hardware: this.extractValue(match[2], 'hardware'),
        os: this.extractValue(match[2], 'os'),
      });
    }

    // Parse physical links
    const linkRegex = /physical_link\s+"([^"]+)"\s*{([^}]+)}/g;
    while ((match = linkRegex.exec(this.code)) !== null) {
      const from = this.extractValue(match[2], 'from') || '';
      const to = this.extractValue(match[2], 'to') || '';
      links.push({
        id: `PL-${links.length + 1}`,
        from,
        to,
        protocol: this.extractValue(match[2], 'protocol') || 'unknown',
        bandwidth: this.extractValue(match[2], 'bandwidth'),
      });
    }

    return { nodes, components, links, deployment };
  }

  private parseEPBS(): EPBSStructure {
    const products: Product[] = [];
    const configurationItems: ConfigurationItem[] = [];
    const breakdown: ProductHierarchy[] = [];

    // Parse products
    const prodRegex = /product\s+"([^"]+)"\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = prodRegex.exec(this.code)) !== null) {
      products.push({
        id: `EPBS-${products.length + 1}`,
        name: match[1],
        version: match[2],
        type: 'system',
      });
    }

    return { products, configurationItems, breakdown };
  }

  private parseRequirements(): RequirementsModel {
    const requirements: Requirement[] = [];
    const traces: TraceLink[] = [];
    const verifications: Verification[] = [];

    // Parse requirements
    const reqRegex = /requirement\s+(\w+)\s*{([^}]+)}/g;
    let match;
    while ((match = reqRegex.exec(this.code)) !== null) {
      requirements.push({
        id: match[1],
        name: match[1],
        text: this.extractValue(match[2], 'text') || '',
        type: (this.extractValue(match[2], 'type') as any) || 'functional',
        safetyLevel: this.extractValue(match[2], 'safety_level'),
        priority: (this.extractValue(match[2], 'priority') as any) || 'medium',
      });
    }

    return { requirements, traces, verifications };
  }

  private parseCrossCutting(): CrossCuttingConcerns {
    const hazards: Hazard[] = [];
    const fmea: FMEA[] = [];
    const sequences: Sequence[] = [];

    // Parse FMEA
    const fmeaRegex = /fmea\s+"([^"]+)"\s*{([^}]+)}/g;
    let match;
    while ((match = fmeaRegex.exec(this.code)) !== null) {
      const body = match[2];
      fmea.push({
        id: `FMEA-${fmea.length + 1}`,
        component: match[1],
        failureMode: this.extractValue(body, 'failure_mode') || '',
        effect: this.extractValue(body, 'effect') || '',
        severity: parseInt(this.extractValue(body, 'severity') || '5'),
        occurrence: parseInt(this.extractValue(body, 'occurrence') || '5'),
        detection: parseInt(this.extractValue(body, 'detection') || '5'),
        rpn: 0,
      });
    }

    // Calculate RPN
    fmea.forEach(f => {
      f.rpn = f.severity * f.occurrence * f.detection;
    });

    return {
      safety: { hazards, fmea, fta: [] },
      modesStates: { modes: [], states: [], transitions: [] },
      sequences,
      interfaces: [],
    };
  }

  private extractValue(text: string, key: string): string | undefined {
    const regex = new RegExp(`${key}:\\s*"([^"]+)"`);
    const match = regex.exec(text);
    return match ? match[1] : undefined;
  }

  private extractArray(text: string, key: string): string[] {
    const regex = new RegExp(`${key}\\s+"([^"]+)"`, 'g');
    const results: string[] = [];
    let match;
    while ((match = regex.exec(text)) !== null) {
      results.push(match[1]);
    }
    return results;
  }
}

// Parse ArcLang code to Arcadia model
export function parseArcLangCode(code: string): ArcadiaModel {
  const parser = new ArcadiaParser(code);
  return parser.parse();
}

// Analyze code and return statistics
export function analyzeArcadiaModel(code: string) {
  const parser = new ArcadiaParser(code);
  const model = parser.parse();

  return {
    operational: {
      actors: model.operational.actors.length,
      activities: model.operational.activities.length,
      total: model.operational.actors.length + model.operational.activities.length,
    },
    system: {
      functions: model.system.functions.length,
      capabilities: model.system.capabilities.length,
      total: model.system.functions.length + model.system.capabilities.length,
    },
    logical: {
      components: model.logical.components.length,
      functions: model.logical.functions.length,
      total: model.logical.components.length + model.logical.functions.length,
    },
    physical: {
      nodes: model.physical.nodes.length,
      links: model.physical.links.length,
      total: model.physical.nodes.length + model.physical.links.length,
    },
    epbs: {
      products: model.epbs.products.length,
      configItems: model.epbs.configurationItems.length,
      total: model.epbs.products.length + model.epbs.configurationItems.length,
    },
    requirements: {
      requirements: model.requirements.requirements.length,
      traces: model.requirements.traces.length,
      total: model.requirements.requirements.length,
    },
    crossCutting: {
      fmea: model.crossCutting.safety.fmea.length,
      sequences: model.crossCutting.sequences.length,
      total: model.crossCutting.safety.fmea.length + model.crossCutting.sequences.length,
    },
    grandTotal:
      model.operational.actors.length +
      model.operational.activities.length +
      model.system.functions.length +
      model.logical.components.length +
      model.physical.nodes.length +
      model.requirements.requirements.length,
  };
}
