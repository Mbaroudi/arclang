import { ArcLangCompiler } from './compiler'

export class ArcLangParser {
  private compiler: ArcLangCompiler

  constructor() {
    this.compiler = new ArcLangCompiler()
  }

  async parseToOperationalModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      console.log('Operational parse failed:', result.errors)
      return null
    }

    const actors = result.diagram.nodes.filter((n: any) => n.type === 'actor')
    const entities = result.diagram.nodes.filter((n: any) => n.type === 'operational_entity' || n.type === 'entity')
    const operationalActivities = result.diagram.nodes.filter((n: any) => n.type === 'operational_activity')
    
    const activities = operationalActivities.map((activity: any) => ({
      id: activity.id,
      name: activity.label,
      performed_by: activity.performed_by || null,
      category: 'operational',
      icon: 'activity',
      color: '#FFF2CC',
      sub_activities: [],
      attributes: {
        description: activity.description || `${activity.label} activity`
      }
    }))
    
    console.log(`Operational model: ${actors.length} actors, ${entities.length} entities, ${activities.length} activities`)
    
    const activityIds = new Set(activities.map((a: any) => a.id))
    
    return {
      name: 'Operational Context',
      actors: actors.map((actor: any) => ({
        id: actor.id,
        name: actor.label,
        description: actor.description || `${actor.label} actor`,
        category: 'External' as const,
        color: null,
        icon: null
      })),
      entities: entities.length > 0 ? entities.map((ent: any) => ({
        id: ent.id,
        name: ent.label,
        description: ent.description || `${ent.label} entity`,
        category: 'Business' as const,
        color: null,
        icon: null
      })) : [],
      capabilities: [],
      activities: activities,
      exchanges: result.diagram.edges
        .filter((e: any) => 
          (e.type === 'connects' || e.type === 'implements') &&
          (activityIds.has(e.source) && activityIds.has(e.target))
        )
        .map((edge: any, idx: number) => ({
          id: edge.id || `exc-${idx}`,
          from: edge.source,
          to: edge.target,
          label: edge.label || `Exchange ${idx + 1}`,
          exchanged_items: []
        })),
      capability_associations: []
    }
  }

  async parseToFunctionalModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const allFunctions = result.diagram.nodes.filter((n: any) => n.type === 'function')
    
    if (allFunctions.length === 0) {
      return null
    }

    const edges = result.diagram.edges
    const functionIds = new Set(allFunctions.map((f: any) => f.id))
    
    const functionEdges = edges.filter((e: any) => 
      (e.type === 'connects' || e.type === 'implements') &&
      functionIds.has(e.source) && 
      functionIds.has(e.target)
    )

    const buildFunctionHierarchy = (funcs: any[], parentId: string | null = null): any[] => {
      return funcs
        .filter(f => f.parent === parentId)
        .map(func => ({
          id: func.id,
          name: func.label,
          category: func.category || 'System' as const,
          color: null,
          icon: null,
          ports: [
            { name: 'in1', direction: 'In' as const, port_type: 'Data' as const, data_type: 'any' },
            { name: 'out1', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'any' }
          ],
          sub_functions: buildFunctionHierarchy(funcs, func.id),
          attributes: {
            description: func.description || `${func.label} function`,
            parent_component: func.parent || null
          }
        }))
    }

    const topLevelFunctions = buildFunctionHierarchy(allFunctions, null)

    return {
      name: 'Functional Architecture',
      requirements: [],
      functions: topLevelFunctions,
      components: [],
      external_actors: [],
      functional_exchanges: functionEdges.map((edge: any) => ({
        from_port: `${edge.source}.out1`,
        to_port: `${edge.target}.in1`,
        data_type: edge.label || 'Data',
        label: edge.label || null
      }))
    }
  }

  async parseToComponentModel(code: string) {
    console.log(`[parseToComponentModel] Called with code length: ${code.length}`)
    console.log(`[parseToComponentModel] Code preview:`, code.substring(0, 100))
    
    const compileResult = await this.compiler.compile(code)
    
    console.log(`[parseToComponentModel] Compiler returned success=${compileResult.success}, hasDiagram=${!!compileResult.diagram}`)
    if (compileResult.diagram) {
      console.log(`[parseToComponentModel] Diagram has ${compileResult.diagram.nodes?.length || 0} nodes, ${compileResult.diagram.edges?.length || 0} edges`)
    }
    
    if (!compileResult.success || !compileResult.diagram) {
      console.log('[parseToComponentModel] Compile failed or no diagram')
      return null
    }

    console.log(`[parseToComponentModel] Total nodes: ${compileResult.diagram.nodes.length}`)
    console.log(`[parseToComponentModel] Node types:`, compileResult.diagram.nodes.map((n: any) => n.type))
    
    const components = compileResult.diagram.nodes.filter((n: any) => n.type === 'component')
    
    console.log(`[parseToComponentModel] Found ${components.length} components`)
    
    if (components.length === 0) {
      return null
    }

    const edges = compileResult.diagram.edges || []
    
    console.log(`[parseToComponentModel] compileResult.diagram.edges type:`, typeof compileResult.diagram.edges)
    console.log(`[parseToComponentModel] compileResult.diagram.edges isArray:`, Array.isArray(compileResult.diagram.edges))
    console.log(`[parseToComponentModel] compileResult.diagram.edges value:`, compileResult.diagram.edges)
    console.log(`[parseToComponentModel] Total edges: ${edges.length}`)
    if (edges.length > 0) {
      console.log(`[parseToComponentModel] Edge types:`, edges.map((e: any) => e.type))
      console.log(`[parseToComponentModel] Edges:`, edges.map((e: any) => `${e.source}->${e.target} (${e.type})`))
    }
    
    // Extract connections between components for component_exchanges
    const componentIds = new Set(components.map((c: any) => c.id))
    console.log(`[parseToComponentModel] Component IDs:`, Array.from(componentIds))
    
    const componentConnections = edges.filter((e: any) => 
      (e.type === 'connects' || e.type === 'implements') && 
      componentIds.has(e.source) && 
      componentIds.has(e.target)
    )
    
    console.log(`[parseToComponentModel] Found ${componentConnections.length} connections between components`)

    const model = {
      name: 'Logical Architecture',
      components: components.map((comp: any) => ({
        id: comp.id,
        name: comp.label,
        component_type: 'Logical',
        color: null,
        sub_components: [],
        allocated_functions: [],
        ports: [],
        functions: [],
        interfaces_in: [],
        interfaces_out: [],
        attributes: {
          description: comp.description || `${comp.label} component`,
          safety_level: comp.safetyLevel || null
        }
      })),
      interfaces: [],
      component_exchanges: componentConnections.map((e: any) => ({
        from_port: `${e.source}.out`,
        to_port: `${e.target}.in`,
        exchange_item: e.label || 'Data',
        label: e.label || null
      })),
      unallocated_functions: []
    }
    
    // Backward compatibility: also add connections field if needed
    const connections = edges
      .filter((e: any) => 
        components.some((c: any) => c.id === e.source) &&
        components.some((c: any) => c.id === e.target)
      )
      .map((edge: any) => ({
        id: edge.id,
        from_component: edge.source,
        to_component: edge.target,
        interface_name: edge.label || 'IData',
        protocol: 'Sync',
        description: edge.label || `Connection from ${edge.source} to ${edge.target}`
      }))
    
    if (connections.length > 0) {
      (model as any).connections = connections
    }
    
    console.log('[parseToComponentModel] Returning model with', model.components.length, 'components')
    
    return model
  }

  async parseToSequenceModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const participants = result.diagram.nodes.filter((n: any) => 
      n.type === 'component' || n.type === 'actor' || n.type === 'function'
    )
    
    if (participants.length < 2 || result.diagram.edges.length === 0) {
      return null
    }

    const interactions = result.diagram.edges.map((edge: any, idx: number) => ({
      id: `msg-${idx + 1}`,
      from: edge.source,
      to: edge.target,
      message: edge.label || `${edge.type}`,
      message_type: 'Sync' as const,
      order: idx + 1,
      description: edge.label || undefined
    }))

    return {
      scenario_name: 'Generated Scenario',
      participants: participants.map((p: any) => ({
        id: p.id,
        name: p.label,
        type: p.type === 'actor' ? 'Actor' as const : 
              p.type === 'component' ? 'Component' as const : 'Function' as const,
        description: p.description || undefined
      })),
      interactions
    }
  }

  async parseToStateMachineModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const states = result.diagram.nodes.filter((n: any) => n.type === 'state' || n.type === 'mode')
    
    if (states.length === 0) {
      return null
    }

    return {
      states: states.map((state: any) => ({
        id: state.id,
        name: state.label,
        type: 'State' as const,
        entry_action: state.entry_action || null,
        exit_action: state.exit_action || null,
        description: state.description || null
      })),
      transitions: result.diagram.edges
        .filter((e: any) => 
          states.some((s: any) => s.id === e.source) &&
          states.some((s: any) => s.id === e.target)
        )
        .map((edge: any) => ({
          id: edge.id,
          from_state: edge.source,
          to_state: edge.target,
          trigger: edge.label || edge.trigger || null,
          guard: edge.guard || null,
          action: edge.action || null
        })),
      initial_state: states[0]?.id || 'idle',
      final_states: states.filter((s: any) => s.final).map((s: any) => s.id)
    }
  }

  async parseToPhysicalModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const nodes = result.diagram.nodes.filter((n: any) => n.type === 'node' || n.layer === 'physical')
    const components = result.diagram.nodes.filter((n: any) => n.type === 'component')
    
    if (nodes.length === 0 && components.length === 0) {
      return null
    }

    return {
      nodes: nodes.map((node: any) => ({
        id: node.id,
        name: node.label,
        node_type: 'Hardware' as const,
        description: node.description || `${node.label} node`,
        color: null,
        icon: null
      })),
      components: components.slice(0, 3).map((comp: any) => ({
        id: comp.id,
        name: comp.label,
        description: comp.description || undefined
      })),
      deployments: []
    }
  }

  async parseToClassModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const components = result.diagram.nodes.filter((n: any) => n.type === 'component')
    
    if (components.length === 0) {
      return null
    }
    
    return {
      classes: components.map((comp: any) => ({
        id: comp.id,
        name: comp.label,
        stereotype: 'Component',
        attributes: [
          { name: 'id', type: 'string', visibility: 'private' },
          { name: 'status', type: 'Status', visibility: 'protected' }
        ],
        methods: [
          { name: 'initialize', return_type: 'void', visibility: 'public', parameters: [] },
          { name: 'execute', return_type: 'Result', visibility: 'public', parameters: [] }
        ],
        description: comp.description || undefined
      })),
      relationships: result.diagram.edges
        .filter((e: any) => e.type === 'implements' || e.type === 'connects')
        .map((edge: any) => ({
          id: edge.id,
          from_class: edge.source,
          to_class: edge.target,
          type: edge.type === 'implements' ? 'Implementation' as const : 'Association' as const,
          label: edge.label || undefined
        }))
    }
  }

  async parseToTreeModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const allNodes = result.diagram.nodes
    const rootNode = allNodes[0]

    if (!rootNode) {
      return { root: null, nodes: [], hierarchy: [] }
    }

    return {
      root: {
        id: rootNode.id,
        name: rootNode.label,
        type: rootNode.type || 'component',
        description: rootNode.description || undefined
      },
      nodes: allNodes.map((node: any) => ({
        id: node.id,
        name: node.label,
        type: node.type || 'component',
        parent_id: node.parent || null,
        description: node.description || undefined
      })),
      hierarchy: result.diagram.edges
        .filter((e: any) => e.type === 'contains' || e.source === rootNode.id)
        .map((edge: any) => ({
          parent_id: edge.source,
          child_id: edge.target
        }))
    }
  }

  async parseToCapabilityModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const requirements = result.diagram.nodes.filter((n: any) => n.type === 'requirement')
    const components = result.diagram.nodes.filter((n: any) => n.type === 'component')

    return {
      capabilities: requirements.slice(0, 5).map((req: any, idx: number) => ({
        id: req.id,
        name: req.label,
        description: req.description || `${req.label} capability`,
        level: idx === 0 ? 'Strategic' as const : 'Operational' as const,
        color: null,
        icon: null
      })),
      capability_realizations: components.slice(0, 3).map((comp: any) => ({
        id: `real-${comp.id}`,
        capability_id: requirements[0]?.id || 'cap-1',
        realized_by: comp.id,
        description: `${comp.label} realizes capability`
      }))
    }
  }

  async parseToFunctionalChainModel(code: string) {
    const result = await this.compiler.compile(code)
    if (!result.success || !result.diagram) {
      return null
    }

    const functions = result.diagram.nodes.filter((n: any) => 
      n.type === 'function' && n.layer === 'logical'
    )
    const functionIds = new Set(functions.map((f: any) => f.id))
    
    console.log(`[parseToFunctionalChain] Found ${functions.length} logical functions:`, Array.from(functionIds))
    console.log(`[parseToFunctionalChain] Total edges: ${result.diagram.edges.length}`)
    
    const exchanges = result.diagram.edges.filter((e: any) => 
      (e.type === 'connects' || e.type === 'implements') &&
      functionIds.has(e.source) &&
      functionIds.has(e.target)
    )
    
    console.log(`[parseToFunctionalChain] Found ${exchanges.length} exchanges between logical functions`)
    if (exchanges.length > 0) {
      console.log(`[parseToFunctionalChain] Sample exchange:`, exchanges[0])
    }

    return {
      functions: functions.map((func: any) => ({
        id: func.id,
        name: func.label,
        category: 'System' as const,
        color: null,
        icon: null,
        ports: [
          { name: 'in1', direction: 'In' as const, port_type: 'Data' as const, data_type: 'any' },
          { name: 'out1', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'any' }
        ],
        sub_functions: [],
        attributes: {}
      })),
      components: [],
      external_actors: [],
      functional_exchanges: exchanges.map((edge: any) => ({
        from_port: `${edge.source}.out1`,
        to_port: `${edge.target}.in1`,
        data_type: 'data',
        label: edge.label || `${edge.source}_to_${edge.target}`
      }))
    }
  }

  private extractPorts(code: string, functionId: string): any[] {
    const ports = []
    const portRegex = new RegExp(`function\\s+"[^"]+.*?{[^}]*id:\\s*"${functionId}"[^}]*}`, 'gs')
    const match = portRegex.exec(code)
    
    if (match) {
      const funcBody = match[0]
      const inPortMatch = funcBody.match(/input:\s*\[([^\]]+)\]/)
      const outPortMatch = funcBody.match(/output:\s*\[([^\]]+)\]/)
      
      if (inPortMatch) {
        const inputs = inPortMatch[1].split(',').map(s => s.trim().replace(/"/g, ''))
        inputs.forEach((inp, idx) => {
          ports.push({
            name: inp || `in${idx + 1}`,
            direction: 'In' as const,
            port_type: 'Data' as const,
            data_type: 'any'
          })
        })
      }
      
      if (outPortMatch) {
        const outputs = outPortMatch[1].split(',').map(s => s.trim().replace(/"/g, ''))
        outputs.forEach((out, idx) => {
          ports.push({
            name: out || `out${idx + 1}`,
            direction: 'Out' as const,
            port_type: 'Data' as const,
            data_type: 'any'
          })
        })
      }
    }
    
    if (ports.length === 0) {
      ports.push(
        { name: 'in1', direction: 'In' as const, port_type: 'Data' as const, data_type: 'any' },
        { name: 'out1', direction: 'Out' as const, port_type: 'Data' as const, data_type: 'any' }
      )
    }
    
    return ports
  }

  async parseCode(code: string, diagramType: string): Promise<any> {
    switch (diagramType) {
      case 'operational':
        return await this.parseToOperationalModel(code)
      case 'functional':
        return await this.parseToFunctionalModel(code)
      case 'component':
        return await this.parseToComponentModel(code)
      case 'sequence':
        return await this.parseToSequenceModel(code)
      case 'state-machine':
        return await this.parseToStateMachineModel(code)
      case 'physical':
        return await this.parseToPhysicalModel(code)
      case 'class':
        return await this.parseToClassModel(code)
      case 'tree':
        return await this.parseToTreeModel(code)
      case 'capability':
        return await this.parseToCapabilityModel(code)
      case 'functional-chain':
        return await this.parseToFunctionalChainModel(code)
      default:
        return null
    }
  }
}
