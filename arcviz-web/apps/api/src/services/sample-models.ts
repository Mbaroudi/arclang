/**
 * Sample models for testing diagram generation
 */

export const sampleOperationalModel = {
  name: 'Vehicle Control System',
  actors: [],
  entities: [
    {
      id: 'actor-driver',
      name: 'Driver',
      entity_type: 'Actor' as const,
      icon: 'actor',
      description: 'Vehicle driver',
      attributes: {},
    },
    {
      id: 'actor-vehicle',
      name: 'Vehicle System',
      entity_type: 'System' as const,
      icon: 'system',
      description: 'Main vehicle system',
      attributes: {},
    },
    {
      id: 'actor-cloud',
      name: 'Cloud Backend',
      entity_type: 'Environment' as const,
      icon: 'cloud',
      description: 'External cloud services',
      attributes: {},
    },
  ],
  capabilities: [],
  activities: [
    {
      id: 'act-1',
      name: 'Start Engine',
      performed_by: 'actor-driver',
      category: 'user',
      icon: 'action',
      color: '#FFF2CC',
      sub_activities: [],
      attributes: {},
    },
    {
      id: 'act-2',
      name: 'Enable ACC',
      performed_by: 'actor-driver',
      category: 'user',
      icon: 'action',
      color: '#FFF2CC',
      sub_activities: [],
      attributes: {},
    },
    {
      id: 'act-3',
      name: 'Monitor Speed',
      performed_by: 'actor-vehicle',
      category: 'system',
      icon: 'process',
      color: '#D5E8D4',
      sub_activities: [],
      attributes: {},
    },
    {
      id: 'act-4',
      name: 'Adjust Speed',
      performed_by: 'actor-vehicle',
      category: 'system',
      icon: 'process',
      color: '#D5E8D4',
      sub_activities: [],
      attributes: {},
    },
    {
      id: 'act-5',
      name: 'Send Telemetry',
      performed_by: 'actor-vehicle',
      category: 'system',
      icon: 'process',
      color: '#D5E8D4',
      sub_activities: [],
      attributes: {},
    },
  ],
  exchanges: [
    {
      from: 'act-1',
      to: 'act-2',
      data_type: 'signal',
      label: 'Engine Ready',
      protocol: null,
      attributes: {},
    },
    {
      from: 'act-2',
      to: 'act-3',
      data_type: 'signal',
      label: 'ACC Enabled',
      protocol: null,
      attributes: {},
    },
    {
      from: 'act-3',
      to: 'act-4',
      data_type: 'data',
      label: 'Speed Data',
      protocol: null,
      attributes: {},
    },
    {
      from: 'act-4',
      to: 'act-5',
      data_type: 'command',
      label: 'Control Commands',
      protocol: null,
      attributes: {},
    },
  ],
  capability_associations: [],
}

export const sampleFunctionalModel = {
  name: 'Adaptive Cruise Control',
  requirements: [],
  functions: [
    {
      id: 'fn-1',
      name: 'Sense Distance',
      category: 'System' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'out-1', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'float' },
      ],
      sub_functions: [],
      attributes: {},
    },
    {
      id: 'fn-2',
      name: 'Calculate Speed',
      category: 'System' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'in-2', direction: 'In' as const, port_type: 'Data' as const, data_type: 'float' },
        { name: 'out-2', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'float' },
      ],
      sub_functions: [],
      attributes: {},
    },
    {
      id: 'fn-3',
      name: 'Control Throttle',
      category: 'Control' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'in-3', direction: 'In' as const, port_type: 'Data' as const, data_type: 'float' },
        { name: 'out-3', direction: 'Out' as const, port_type: 'Control' as const, data_type: 'command' },
      ],
      sub_functions: [],
      attributes: {},
    },
  ],
  components: [],
  external_actors: [],
  functional_exchanges: [
    {
      from_port: 'fn-1.out-1',
      to_port: 'fn-2.in-2',
      data_type: 'float',
      label: 'distance',
    },
    {
      from_port: 'fn-2.out-2',
      to_port: 'fn-3.in-3',
      data_type: 'float',
      label: 'target_speed',
    },
  ],
}

export const sampleComponentModel = {
  name: 'ACC System Architecture',
  components: [
    {
      id: 'comp-1',
      name: 'RadarSubsystem',
      component_type: 'Sensor',
      color: null,
      sub_components: [],
      allocated_functions: [],
      ports: [
        { name: 'port-1', direction: 'Out' as const, interface_type: 'IRadarData' },
      ],
      functions: [],
      interfaces_in: [],
      interfaces_out: [
        { name: 'IRadarData', protocol: 'CAN', format: null, attributes: {} },
      ],
      attributes: {},
    },
    {
      id: 'comp-2',
      name: 'ACCController',
      component_type: 'Controller',
      color: null,
      sub_components: [],
      allocated_functions: [],
      ports: [
        { name: 'port-2', direction: 'In' as const, interface_type: 'IRadarData' },
        { name: 'port-3', direction: 'Out' as const, interface_type: 'ISpeedControl' },
      ],
      functions: [],
      interfaces_in: [
        { name: 'IRadarData', protocol: 'CAN', format: null, attributes: {} },
      ],
      interfaces_out: [
        { name: 'ISpeedControl', protocol: 'Ethernet', format: null, attributes: {} },
      ],
      attributes: {},
    },
  ],
  interfaces: [
    {
      name: 'IRadarData',
      from: 'comp-1',
      to: 'comp-2',
      attributes: {},
    },
  ],
  component_exchanges: [
    {
      from_port: 'comp-1.port-1',
      to_port: 'comp-2.port-2',
      exchange_item: 'RadarData',
      label: 'IRadarData',
    },
  ],
  unallocated_functions: [],
}

export const sampleSequenceModel = {
  name: 'ACC Activation Scenario',
  participants: [
    { id: 'p1', name: 'Driver', participant_type: 'Actor' as const, lifeline_color: '#FFD700' },
    { id: 'p2', name: 'HMI', participant_type: 'Component' as const, lifeline_color: '#87CEEB' },
    { id: 'p3', name: 'ACC_Controller', participant_type: 'Component' as const, lifeline_color: '#90EE90' },
    { id: 'p4', name: 'Radar', participant_type: 'Component' as const, lifeline_color: '#FFB6C1' },
  ],
  messages: [
    {
      from: 'p1',
      to: 'p2',
      label: 'Press ACC Button',
      message_type: 'Synchronous' as const,
      activation: true,
      timing: null,
      params: null,
    },
    {
      from: 'p2',
      to: 'p3',
      label: 'Enable ACC',
      message_type: 'Asynchronous' as const,
      activation: false,
      timing: null,
      params: null,
    },
    {
      from: 'p3',
      to: 'p4',
      label: 'Start Monitoring',
      message_type: 'Synchronous' as const,
      activation: true,
      timing: null,
      params: null,
    },
    {
      from: 'p4',
      to: 'p3',
      label: 'Distance Data',
      message_type: 'Return' as const,
      activation: false,
      timing: null,
      params: null,
    },
  ],
  fragments: [],
  timing_constraints: [],
}

export const sampleStateMachineModel = {
  name: 'ACC State Machine',
  initial_state: 'Off',
  states: [
    {
      name: 'Off',
      entry_actions: [],
      exit_actions: [],
      internal_transitions: [],
      sub_states: [],
      color: null,
    },
    {
      name: 'Standby',
      entry_actions: ['initialize'],
      exit_actions: [],
      internal_transitions: [],
      sub_states: [],
      color: null,
    },
    {
      name: 'Active',
      entry_actions: ['start_control'],
      exit_actions: ['stop_control'],
      internal_transitions: [],
      sub_states: [],
      color: null,
    },
    {
      name: 'Error',
      entry_actions: ['log_error'],
      exit_actions: [],
      internal_transitions: [],
      sub_states: [],
      color: '#FF0000',
    },
  ],
  transitions: [
    {
      from: 'Off',
      to: 'Standby',
      trigger: 'power_on',
      guard: 'system_ready',
      action: 'initialize',
      timing: null,
      priority: null,
    },
    {
      from: 'Standby',
      to: 'Active',
      trigger: 'enable_acc',
      guard: 'speed_valid',
      action: 'start_control',
      timing: null,
      priority: null,
    },
    {
      from: 'Active',
      to: 'Standby',
      trigger: 'disable_acc',
      guard: null,
      action: 'stop_control',
      timing: null,
      priority: null,
    },
    {
      from: 'Active',
      to: 'Error',
      trigger: 'sensor_fault',
      guard: null,
      action: 'log_error',
      timing: null,
      priority: null,
    },
  ],
}

export const samplePhysicalModel = {
  name: 'Physical Deployment',
  nodes: [
    {
      id: 'node1',
      name: 'Central_ECU',
      node_type: 'Hardware' as const,
      color: null,
      processor: 'ARM Cortex-A53',
      memory: '4GB',
      behavior_components: [
        { id: 'bc1', name: 'ACCController', allocated_functions: [], color: null },
        { id: 'bc2', name: 'HMI', allocated_functions: [], color: null },
      ],
      hardware_components: [],
      deployments: [
        { component: 'ACCController', attributes: {} },
        { component: 'HMI', attributes: {} },
      ],
      attributes: {},
    },
    {
      id: 'node2',
      name: 'Radar_ECU',
      node_type: 'Hardware' as const,
      color: null,
      processor: 'Dedicated DSP',
      memory: '512MB',
      behavior_components: [
        { id: 'bc3', name: 'RadarSubsystem', allocated_functions: [], color: null },
      ],
      hardware_components: [],
      deployments: [
        { component: 'RadarSubsystem', attributes: {} },
      ],
      attributes: {},
    },
  ],
  links: [
    {
      from: 'node1',
      to: 'node2',
      protocol: 'CAN',
      bandwidth: '500kbps',
      color: null,
      connections: [],
      attributes: {},
    },
  ],
  physical_exchanges: [],
}

export const sampleClassModel = {
  name: 'Vehicle Data Model',
  exchange_items: [
    {
      name: 'VehicleSpeed',
      stereotype: 'data',
      attributes: [
        { name: 'current_speed', attr_type: 'float', default_value: null, enumeration: null },
        { name: 'timestamp', attr_type: 'uint64', default_value: null, enumeration: null },
      ],
    },
    {
      name: 'RadarData',
      stereotype: 'data',
      attributes: [
        { name: 'distance', attr_type: 'float', default_value: null, enumeration: null },
        { name: 'relative_speed', attr_type: 'float', default_value: null, enumeration: null },
      ],
    },
  ],
  data_types: [
    {
      name: 'SpeedUnit',
      base_type: 'enum',
      enumeration_values: [
        { name: 'KPH', value: null },
        { name: 'MPH', value: null },
      ],
    },
  ],
}

export const sampleTreeModel = {
  name: 'Function Hierarchy',
  systems: [
    {
      name: 'Vehicle Control',
      subsystems: [
        {
          name: 'Speed Control',
          items: [
            { name: 'ACC', attributes: {} },
            { name: 'Cruise Control', attributes: {} },
          ],
          attributes: {},
        },
        {
          name: 'Safety',
          items: [
            { name: 'AEB', attributes: {} },
            { name: 'LKA', attributes: {} },
          ],
          attributes: {},
        },
      ],
      attributes: {},
    },
  ],
}

export const sampleCapabilityModel = {
  name: 'System Capabilities',
  actors: [],
  entities: [],
  capabilities: [
    {
      id: 'cap1',
      name: 'Autonomous Driving',
      level: 'Mission' as const,
      color: null,
      stereotype: null,
      children: [
        {
          id: 'cap2',
          name: 'Adaptive Cruise Control',
          level: 'Capability' as const,
          color: null,
          stereotype: null,
          children: [
            {
              id: 'cap3',
              name: 'Distance Sensing',
              level: 'SubCapability' as const,
              color: null,
              stereotype: null,
              children: [],
              attributes: {},
            },
          ],
          attributes: {},
        },
      ],
      attributes: {},
    },
  ],
  activities: [],
  exchanges: [],
  capability_associations: [
    {
      from: 'cap1',
      to: 'cap2',
      association_type: 'includes',
      label: null,
    },
    {
      from: 'cap2',
      to: 'cap3',
      association_type: 'includes',
      label: null,
    },
  ],
}

export const sampleFunctionalChainModel = {
  name: 'Emergency Stop Chain',
  requirements: [],
  functions: [
    {
      id: 'fc-1',
      name: 'Detect Obstacle',
      category: 'Environmental' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'out1', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'bool' },
      ],
      sub_functions: [],
      attributes: {},
    },
    {
      id: 'fc-2',
      name: 'Calculate Brake Force',
      category: 'System' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'in2', direction: 'In' as const, port_type: 'Data' as const, data_type: 'bool' },
        { name: 'out2', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'float' },
      ],
      sub_functions: [],
      attributes: {},
    },
    {
      id: 'fc-3',
      name: 'Apply Brakes',
      category: 'Control' as const,
      color: null,
      icon: null,
      ports: [
        { name: 'in3', direction: 'In' as const, port_type: 'Data' as const, data_type: 'float' },
        { name: 'out3', direction: 'Out' as const, port_type: 'Control' as const, data_type: 'bool' },
      ],
      sub_functions: [],
      attributes: {},
    },
  ],
  components: [],
  external_actors: [],
  functional_exchanges: [
    {
      from_port: 'fc-1.out1',
      to_port: 'fc-2.in2',
      data_type: 'bool',
      label: 'obstacle_detected',
    },
    {
      from_port: 'fc-2.out2',
      to_port: 'fc-3.in3',
      data_type: 'float',
      label: 'brake_force',
    },
  ],
}

export function getSampleModel(diagramType: string): any {
  const models: Record<string, any> = {
    operational: sampleOperationalModel,
    functional: sampleFunctionalModel,
    component: sampleComponentModel,
    sequence: sampleSequenceModel,
    'state-machine': sampleStateMachineModel,
    physical: samplePhysicalModel,
    class: sampleClassModel,
    tree: sampleTreeModel,
    capability: sampleCapabilityModel,
    'functional-chain': sampleFunctionalChainModel,
  }

  return models[diagramType] || null
}
