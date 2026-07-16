export interface Actor {
  id: string;
  name: string;
  description?: string;
  category?: string;
}

export interface OperationalActivity {
  id: string;
  name: string;
  performed_by?: string;
  description?: string;
}

export interface Capability {
  id: string;
  name: string;
  activities?: string[];
}

export interface Interaction {
  from: string;
  to: string;
  data?: string;
}

export interface OperationalAnalysis {
  actors: Actor[];
  activities: OperationalActivity[];
  capabilities: Capability[];
  interactions: Interaction[];
}

export interface SystemFunction {
  id: string;
  name: string;
  allocated_to?: string;
  description?: string;
}

export interface SystemAnalysis {
  system: {
    id: string;
    name: string;
  };
  actors: Actor[];
  functions: SystemFunction[];
  interactions: Interaction[];
}

export interface Component {
  id: string;
  name: string;
  provides?: string[];
  requires?: string[];
}

export interface Interface {
  id: string;
  name: string;
}

export interface DataFlow {
  from: string;
  to: string;
  data?: string;
}

export interface LogicalArchitecture {
  components: Component[];
  interfaces: Interface[];
  dataFlows: DataFlow[];
}

export interface PhysicalNode {
  id: string;
  name: string;
  type: 'hardware' | 'software' | 'behavior';
}

export interface PhysicalLink {
  from: string;
  to: string;
  type: string;
}

export interface Deployment {
  component: string;
  node: string;
}

export interface PhysicalArchitecture {
  nodes: PhysicalNode[];
  links: PhysicalLink[];
  deployments: Deployment[];
}

export interface Subsystem {
  id: string;
  name: string;
}

export interface Assembly {
  id: string;
  name: string;
  parent?: string;
}

export interface EPBSComponent {
  id: string;
  name: string;
  parent?: string;
}

export interface EPBSStructure {
  subsystems: Subsystem[];
  assemblies: Assembly[];
  components: EPBSComponent[];
}

export interface Requirement {
  id: string;
  name: string;
  type: 'functional' | 'non-functional';
  priority: 'low' | 'medium' | 'high' | 'critical';
  status: 'draft' | 'approved' | 'implemented' | 'verified';
  text?: string;
  allocated_to?: string[];
  refined_by?: string[];
}

export interface Trace {
  from: string;
  to: string;
}

export interface RequirementsModel {
  requirements: Requirement[];
  traces: Trace[];
}

export interface SecurityPolicy {
  id: string;
  name: string;
  description?: string;
}

export interface SafetyConstraint {
  id: string;
  name: string;
  level?: string;
}

export interface PerformanceMetric {
  id: string;
  name: string;
  target?: string;
}

export interface Dependency {
  from: string;
  to: string;
}

export interface CrossCuttingConcerns {
  securityPolicies: SecurityPolicy[];
  safetyConstraints: SafetyConstraint[];
  performanceMetrics: PerformanceMetric[];
  dependencies: Dependency[];
}

export interface ArcadiaModel {
  operational: OperationalAnalysis;
  system: SystemAnalysis;
  logical: LogicalArchitecture;
  physical: PhysicalArchitecture;
  epbs: EPBSStructure;
  requirements: RequirementsModel;
  crossCutting: CrossCuttingConcerns;
}

export function parseArcLangCode(code: string): ArcadiaModel {
  const lines = code.split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('//'));
  
  const operational: OperationalAnalysis = {
    actors: [],
    activities: [],
    capabilities: [],
    interactions: []
  };
  
  const system: SystemAnalysis = {
    system: { id: 'main_system', name: 'Main System' },
    actors: [],
    functions: [],
    interactions: []
  };
  
  const logical: LogicalArchitecture = {
    components: [],
    interfaces: [],
    dataFlows: []
  };
  
  const physical: PhysicalArchitecture = {
    nodes: [],
    links: [],
    deployments: []
  };
  
  const epbs: EPBSStructure = {
    subsystems: [],
    assemblies: [],
    components: []
  };
  
  const requirements: RequirementsModel = {
    requirements: [],
    traces: []
  };
  
  const crossCutting: CrossCuttingConcerns = {
    securityPolicies: [],
    safetyConstraints: [],
    performanceMetrics: [],
    dependencies: []
  };
  
  let currentBlock: any = null;
  let blockType = '';
  
  for (const line of lines) {
    if (line.match(/^actor\s+"([^"]+)"\s+as\s+(\w+)/)) {
      const match = line.match(/^actor\s+"([^"]+)"\s+as\s+(\w+)/);
      const actor = { id: match![2], name: match![1] };
      operational.actors.push(actor);
      system.actors.push(actor);
    }
    
    else if (line.match(/^operational_activity\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^operational_activity\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      };
      blockType = 'operational_activity';
    }
    
    else if (line.match(/^capability\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^capability\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1],
        activities: []
      };
      blockType = 'capability';
    }
    
    else if (line.match(/^system_function\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^system_function\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      };
      blockType = 'system_function';
    }
    
    else if (line.match(/^component\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^component\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1],
        provides: [],
        requires: []
      };
      blockType = 'component';
    }
    
    else if (line.match(/^interface\s+"([^"]+)"/)) {
      const match = line.match(/^interface\s+"([^"]+)"/);
      logical.interfaces.push({
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      });
    }
    
    else if (line.match(/^physical_node\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^physical_node\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1],
        type: 'hardware'
      };
      blockType = 'physical_node';
    }
    
    else if (line.match(/^requirement\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^requirement\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1],
        name: match![1],
        type: 'functional',
        priority: 'medium',
        status: 'draft'
      };
      blockType = 'requirement';
    }
    
    else if (line.match(/^security_policy\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^security_policy\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      };
      blockType = 'security_policy';
    }
    
    else if (line.match(/^safety_constraint\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^safety_constraint\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      };
      blockType = 'safety_constraint';
    }
    
    else if (line.match(/^performance_metric\s+"([^"]+)"\s*\{/)) {
      const match = line.match(/^performance_metric\s+"([^"]+)"\s*\{/);
      currentBlock = {
        id: match![1].toLowerCase().replace(/\s+/g, '_'),
        name: match![1]
      };
      blockType = 'performance_metric';
    }
    
    else if (line.match(/^deployment\s+(\w+)\s*->\s*(\w+)/)) {
      const match = line.match(/^deployment\s+(\w+)\s*->\s*(\w+)/);
      physical.deployments.push({
        component: match![1],
        node: match![2]
      });
    }
    
    else if (line.match(/^interaction\s+(\w+)\s*->\s*(\w+)\s*:\s*"([^"]+)"/)) {
      const match = line.match(/^interaction\s+(\w+)\s*->\s*(\w+)\s*:\s*"([^"]+)"/);
      const interaction = { from: match![1], to: match![2], data: match![3] };
      operational.interactions.push(interaction);
      system.interactions.push(interaction);
    }
    
    else if (currentBlock && line.includes(':')) {
      const [key, ...valueParts] = line.split(':');
      const value = valueParts.join(':').trim().replace(/[",]/g, '');
      
      if (key.trim() === 'performed_by' && blockType === 'operational_activity') {
        currentBlock.performed_by = value;
      } else if (key.trim() === 'allocated_to') {
        if (Array.isArray(currentBlock.allocated_to)) {
          currentBlock.allocated_to.push(value);
        } else {
          currentBlock.allocated_to = value;
        }
      } else if (key.trim() === 'activities' && blockType === 'capability') {
        currentBlock.activities = value.replace(/[\[\]]/g, '').split(',').map((s: string) => s.trim().replace(/"/g, ''));
      } else if (key.trim() === 'provides' && blockType === 'component') {
        currentBlock.provides = value.replace(/[\[\]]/g, '').split(',').map((s: string) => s.trim().replace(/"/g, ''));
      } else if (key.trim() === 'requires' && blockType === 'component') {
        currentBlock.requires = value.replace(/[\[\]]/g, '').split(',').map((s: string) => s.trim().replace(/"/g, ''));
      } else if (key.trim() === 'type') {
        currentBlock.type = value;
      } else if (key.trim() === 'priority') {
        currentBlock.priority = value;
      } else if (key.trim() === 'status') {
        currentBlock.status = value;
      } else if (key.trim() === 'text') {
        currentBlock.text = value;
      } else if (key.trim() === 'name') {
        currentBlock.name = value;
      } else if (key.trim() === 'description') {
        currentBlock.description = value;
      } else if (key.trim() === 'target') {
        currentBlock.target = value;
      }
    }
    
    else if (line === '}' && currentBlock) {
      switch (blockType) {
        case 'operational_activity':
          operational.activities.push(currentBlock);
          break;
        case 'capability':
          operational.capabilities.push(currentBlock);
          break;
        case 'system_function':
          system.functions.push(currentBlock);
          break;
        case 'component':
          logical.components.push(currentBlock);
          break;
        case 'physical_node':
          physical.nodes.push(currentBlock);
          break;
        case 'requirement':
          requirements.requirements.push(currentBlock);
          break;
        case 'security_policy':
          crossCutting.securityPolicies.push(currentBlock);
          break;
        case 'safety_constraint':
          crossCutting.safetyConstraints.push(currentBlock);
          break;
        case 'performance_metric':
          crossCutting.performanceMetrics.push(currentBlock);
          break;
      }
      currentBlock = null;
      blockType = '';
    }
  }
  
  return {
    operational,
    system,
    logical,
    physical,
    epbs,
    requirements,
    crossCutting
  };
}
