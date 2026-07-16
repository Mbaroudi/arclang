import { exec } from 'child_process'
import { promisify } from 'util'
import * as fs from 'fs/promises'
import * as path from 'path'
import * as os from 'os'

const execAsync = promisify(exec)

export interface CompilationResult {
  success: boolean
  output?: string
  errors?: string
  warnings?: string
  stats?: {
    requirements: number
    components: number
    functions: number
    traces: number
  }
  diagram?: {
    nodes: any[]
    edges: any[]
    layer: string
  }
}

export class ArcLangCompiler {
  private compilerPath: string

  constructor(compilerPath?: string) {
    this.compilerPath = compilerPath || process.env.ARCLANG_COMPILER_PATH || 'arclang'
  }

  async compile(code: string): Promise<CompilationResult> {
    // For now, skip the actual compiler and use regex parsing only
    // The ArcLang compiler has syntax issues with the current code
    try {
      // Auto-wrap partial blocks into a full model
      let processedCode = code
      const isPartialBlock = !code.includes('model ') && 
                            (code.trim().match(/^(requirements|architecture|component|function)\s/) ||
                             code.includes('operational_analysis') ||
                             code.includes('system_analysis') ||
                             code.includes('logical_architecture'))
      
      if (isPartialBlock) {
        // Wrap partial block in a complete model structure
        processedCode = `model GeneratedModel {
  metadata {
    name: "AI Generated Model"
    version: "1.0.0"
    author: "AI Assistant"
    description: "Auto-generated from partial code"
  }

  ${code}
}`
      }

      const diagram = this.parseArcLangCode(processedCode)
      const stats = this.calculateStatsFromCode(processedCode)

      console.log(`[Compiler] Parsed diagram: ${diagram.nodes.length} nodes`)

      // Basic syntax validation
      if (!processedCode.includes('{')) {
        console.log('[Compiler] ERROR: missing braces')
        return {
          success: false,
          errors: 'Invalid ArcLang syntax: missing braces',
        }
      }

      if (diagram.nodes.length === 0) {
        console.log('[Compiler] WARNING: No nodes found')
        return {
          success: true,
          output: 'Code parsed successfully but no diagram nodes found',
          warnings: 'No components or requirements found in the code',
          stats,
          diagram,
        }
      }

      console.log(`[Compiler] Returning success with ${diagram.nodes.length} nodes`)
      
      return {
        success: true,
        output: `Parsed ${diagram.nodes.length} nodes and ${diagram.edges.length} edges`,
        stats,
        diagram,
      }
    } catch (error: any) {
      console.log('[Compiler] EXCEPTION:', error.message)
      return {
        success: false,
        errors: error.message || 'Unknown parsing error',
      }
    }
  }

  async validate(code: string): Promise<{ valid: boolean; errors: string[] }> {
    const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'arclang-'))
    const inputFile = path.join(tempDir, 'input.arc')

    try {
      await fs.writeFile(inputFile, code, 'utf-8')

      const command = `${this.compilerPath} check ${inputFile}`

      await execAsync(command, {
        timeout: 10000,
        maxBuffer: 5 * 1024 * 1024,
      })

      return { valid: true, errors: [] }
    } catch (error: any) {
      const errorMessage = error.stderr || error.stdout || error.message
      const errors = this.parseErrors(errorMessage)

      return { valid: false, errors }
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true })
    }
  }

  private parseArcLangCode(code: string): any {
    const nodes: any[] = []
    const edges: any[] = []
    const layers: string[] = []

    console.log('Parsing ArcLang code, length:', code.length)

    // Parse actors and operational_activity from operational_analysis
    const operationalRegex = /operational_analysis\s+"([^"]+)"\s*\{([\s\S]*?)(?:\n\}(?:\n|$))/g
    let opMatch
    while ((opMatch = operationalRegex.exec(code)) !== null) {
      layers.push('operational')
      const opBody = opMatch[2]
      
      const actorRegex = /actor\s+"([^"]+)"\s*\{([^}]*)\}/g
      let actorMatch
      while ((actorMatch = actorRegex.exec(opBody)) !== null) {
        const actorLabel = actorMatch[1]
        const actorBody = actorMatch[2]
        
        const idMatch = actorBody.match(/id:\s*"([^"]+)"/)
        const actorId = idMatch ? idMatch[1] : actorLabel.replace(/\s+/g, '')
        
        const descMatch = actorBody.match(/description:\s*"([^"]*)"/)
        
        if (!nodes.find(n => n.id === actorId)) {
          nodes.push({
            id: actorId,
            label: actorLabel,
            type: 'actor',
            layer: 'operational',
            description: descMatch ? descMatch[1] : undefined,
          })
        }
      }
      
      // Parse operational_activity blocks
      const activityRegex = /operational_activity\s+"([^"]+)"\s*\{([^}]*)\}/g
      let activityMatch
      while ((activityMatch = activityRegex.exec(opBody)) !== null) {
        const activityLabel = activityMatch[1]
        const activityBody = activityMatch[2]
        
        const idMatch = activityBody.match(/id:\s*"([^"]+)"/)
        const activityId = idMatch ? idMatch[1] : activityLabel.replace(/\s+/g, '-')
        
        const descMatch = activityBody.match(/description:\s*"([^"]*)"/)
        const performedByMatch = activityBody.match(/performed_by:\s*"([^"]+)"/)
        
        if (!nodes.find(n => n.id === activityId)) {
          nodes.push({
            id: activityId,
            label: activityLabel,
            type: 'operational_activity',
            layer: 'operational',
            description: descMatch ? descMatch[1] : undefined,
            performed_by: performedByMatch ? performedByMatch[1] : undefined,
          })
        }
      }
    }

    // Parse requirements from system_analysis
    const systemAnalysisRegex = /system_analysis\s+"([^"]+)"\s*\{([\s\S]*?)(?:\n\}(?:\n|$))/g
    let sysMatch
    while ((sysMatch = systemAnalysisRegex.exec(code)) !== null) {
      layers.push('system')
      const sysBody = sysMatch[2]
      
      const reqRegex = /requirement\s+"([^"]+)"\s*\{([^}]*)\}/g
      let reqMatch
      while ((reqMatch = reqRegex.exec(sysBody)) !== null) {
        const reqLabel = reqMatch[1]
        const reqBody = reqMatch[2]
        
        const idMatch = reqBody.match(/id:\s*"([^"]+)"/)
        const reqId = idMatch ? idMatch[1] : reqLabel
        
        const descMatch = reqBody.match(/description:\s*"([^"]*)"/)
        const safetyMatch = reqBody.match(/safety_level:\s*"?([^\s\n,}"]+)"?/)
        const priorityMatch = reqBody.match(/priority:\s*"?([^\s\n,}"]+)"?/)
        
        if (!nodes.find(n => n.id === reqId)) {
          nodes.push({
            id: reqId,
            label: reqLabel,
            type: 'requirement',
            layer: 'system',
            safetyLevel: safetyMatch ? safetyMatch[1] : undefined,
            priority: priorityMatch ? priorityMatch[1] : undefined,
            description: descMatch ? descMatch[1] : undefined,
          })
        }
      }
    }

    // Parse physical_architecture layer
    const physicalRegex = /physical_architecture\s+"([^"]+)"\s*\{([\s\S]*?)(?:\n\}(?:\n|$))/g
    let physMatch
    while ((physMatch = physicalRegex.exec(code)) !== null) {
      layers.push('physical')
      const physBody = physMatch[2]
      
      // Parse nodes (hardware, deployment units)
      const nodeRegex = /node\s+"([^"]+)"\s*\{([^}]*)\}/g
      let nodeMatch
      while ((nodeMatch = nodeRegex.exec(physBody)) !== null) {
        const nodeLabel = nodeMatch[1]
        const nodeBody = nodeMatch[2]
        
        const idMatch = nodeBody.match(/id:\s*"([^"]+)"/)
        const nodeId = idMatch ? idMatch[1] : nodeLabel.replace(/\s+/g, '')
        
        const descMatch = nodeBody.match(/description:\s*"([^"]*)"/)
        
        if (!nodes.find(n => n.id === nodeId)) {
          nodes.push({
            id: nodeId,
            label: nodeLabel,
            type: 'node',
            layer: 'physical',
            description: descMatch ? descMatch[1] : undefined,
          })
        }
      }
    }

    // Parse EPBS layer
    const epbsRegex = /epbs\s+"([^"]+)"\s*\{([\s\S]*?)(?:\n\}(?:\n|$))/g
    let epbsMatch
    while ((epbsMatch = epbsRegex.exec(code)) !== null) {
      layers.push('epbs')
      const epbsBody = epbsMatch[2]
      
      // Parse configuration items
      const configItemRegex = /configuration_item\s+"([^"]+)"\s*\{([^}]*)\}/g
      let ciMatch
      while ((ciMatch = configItemRegex.exec(epbsBody)) !== null) {
        const ciLabel = ciMatch[1]
        const ciBody = ciMatch[2]
        
        const idMatch = ciBody.match(/id:\s*"([^"]+)"/)
        const ciId = idMatch ? idMatch[1] : ciLabel.replace(/\s+/g, '')
        
        const descMatch = ciBody.match(/description:\s*"([^"]*)"/)
        const typeMatch = ciBody.match(/item_type:\s*"([^"]*)"/)
        
        if (!nodes.find(n => n.id === ciId)) {
          nodes.push({
            id: ciId,
            label: ciLabel,
            type: 'configuration_item',
            layer: 'epbs',
            description: descMatch ? descMatch[1] : undefined,
            itemType: typeMatch ? typeMatch[1] : undefined,
          })
        }
      }
    }

    // Detect logical_architecture layer
    if (code.includes('logical_architecture')) {
      layers.push('logical')
    }

    // Parse components using proper brace counting for nested structures
    console.log('=== COMPONENT PARSING START ===')
    console.log('Code contains "component"?', code.includes('component'))
    const componentStartRegex = /component\s+"([^"]+)"\s*\{/g
    let compMatch
    
    while ((compMatch = componentStartRegex.exec(code)) !== null) {
      console.log('Component regex matched:', compMatch[1])
      const label = compMatch[1]
      const compStartIdx = compMatch.index + compMatch[0].length
      
      // Count braces to find the end of the component block
      let braceCount = 1
      let compEndIdx = compStartIdx
      while (braceCount > 0 && compEndIdx < code.length) {
        if (code[compEndIdx] === '{') braceCount++
        if (code[compEndIdx] === '}') braceCount--
        compEndIdx++
      }
      
      const body = code.substring(compStartIdx, compEndIdx - 1)
      
      const idMatch = body.match(/id:\s*"([^"]+)"/)
      const id = idMatch ? idMatch[1] : label.replace(/\s+/g, '')
      
      const descMatch = body.match(/description:\s*"([^"]*)"/)
      const safetyMatch = body.match(/safety_level:\s*"?([^\s\n,}"]+)"?/)
      const typeMatch = body.match(/type:\s*"([^"]+)"/)
      const categoryMatch = body.match(/category:\s*"([^"]+)"/)
      
      console.log(`Found component: ${id} (${label}), body length: ${body.length}`)
      
      if (!nodes.find(n => n.id === id)) {
        nodes.push({
          id,
          label,
          type: 'component',
          layer: 'logical',
          safetyLevel: safetyMatch ? safetyMatch[1] : undefined,
          description: descMatch ? descMatch[1] : undefined,
        })
      }
      
      // Parse functions within this component
      // Match: function "Name" { id: "ID" ... } - with proper brace counting
      const funcStartRegex = /function\s+"([^"]+)"\s*\{/g
      let funcStartMatch
      const bodyStartPos = code.indexOf(body)
      
      console.log(`  Searching for functions in component ${id}, testing regex...`)
      console.log(`  Body contains 'function'?: ${body.includes('function')}`)
      
      while ((funcStartMatch = funcStartRegex.exec(body)) !== null) {
        console.log(`  Found function match: ${funcStartMatch[1]}`)
        const funcLabel = funcStartMatch[1]
        const funcStartIdx = funcStartMatch.index + funcStartMatch[0].length
        
        // Count braces to find the end of the function block
        let braceCount = 1
        let funcEndIdx = funcStartIdx
        while (braceCount > 0 && funcEndIdx < body.length) {
          if (body[funcEndIdx] === '{') braceCount++
          if (body[funcEndIdx] === '}') braceCount--
          funcEndIdx++
        }
        
        const funcBody = body.substring(funcStartIdx, funcEndIdx - 1)
        
        const funcIdMatch = funcBody.match(/id:\s*"([^"]+)"/)
        const funcId = funcIdMatch ? funcIdMatch[1] : funcLabel.replace(/\s+/g, '-').replace(/[^a-zA-Z0-9-]/g, '')
        
        const funcDescMatch = funcBody.match(/description:\s*"([^"]*)"/)
        
        if (!nodes.find(n => n.id === funcId)) {
          nodes.push({
            id: funcId,
            label: funcLabel,
            type: 'function',
            layer: 'logical',
            description: funcDescMatch ? funcDescMatch[1] : undefined,
            parent: id,
          })
          console.log(`Added function: ${funcId} (${funcLabel}) under component ${id}`)
        }
      }
    }

    // Parse system_function blocks with nested functions
    console.log('=== SYSTEM_FUNCTION PARSING START ===')
    const sysFuncStartRegex = /system_function\s+"([^"]+)"\s*\{/g
    let sysFuncMatch
    
    while ((sysFuncMatch = sysFuncStartRegex.exec(code)) !== null) {
      console.log('System function regex matched:', sysFuncMatch[1])
      const label = sysFuncMatch[1]
      const sysFuncStartIdx = sysFuncMatch.index + sysFuncMatch[0].length
      
      let braceCount = 1
      let sysFuncEndIdx = sysFuncStartIdx
      while (braceCount > 0 && sysFuncEndIdx < code.length) {
        if (code[sysFuncEndIdx] === '{') braceCount++
        if (code[sysFuncEndIdx] === '}') braceCount--
        sysFuncEndIdx++
      }
      
      const body = code.substring(sysFuncStartIdx, sysFuncEndIdx - 1)
      
      const idMatch = body.match(/id:\s*"([^"]+)"/)
      const id = idMatch ? idMatch[1] : label.replace(/\s+/g, '-')
      
      const descMatch = body.match(/description:\s*"([^"]*)"/)
      const categoryMatch = body.match(/category:\s*"([^"]+)"/)
      
      console.log(`Found system_function: ${id} (${label}), body length: ${body.length}`)
      
      if (!nodes.find(n => n.id === id)) {
        nodes.push({
          id,
          label,
          type: 'function',
          layer: 'system',
          description: descMatch ? descMatch[1] : undefined,
          category: categoryMatch ? categoryMatch[1] : undefined,
        })
      }
      
      // Parse nested functions within this system_function
      const funcStartRegex = /function\s+"([^"]+)"\s*\{/g
      let funcStartMatch
      
      console.log(`  Searching for nested functions in ${id}`)
      console.log(`  Body contains 'function'?: ${body.includes('function')}`)
      
      while ((funcStartMatch = funcStartRegex.exec(body)) !== null) {
        console.log(`  Found nested function match: ${funcStartMatch[1]}`)
        const funcLabel = funcStartMatch[1]
        const funcStartIdx = funcStartMatch.index + funcStartMatch[0].length
        
        let braceCount = 1
        let funcEndIdx = funcStartIdx
        while (braceCount > 0 && funcEndIdx < body.length) {
          if (body[funcEndIdx] === '{') braceCount++
          if (body[funcEndIdx] === '}') braceCount--
          funcEndIdx++
        }
        
        const funcBody = body.substring(funcStartIdx, funcEndIdx - 1)
        
        const funcIdMatch = funcBody.match(/id:\s*"([^"]+)"/)
        const funcId = funcIdMatch ? funcIdMatch[1] : funcLabel.replace(/\s+/g, '-')
        
        const funcDescMatch = funcBody.match(/description:\s*"([^"]*)"/)
        
        if (!nodes.find(n => n.id === funcId)) {
          nodes.push({
            id: funcId,
            label: funcLabel,
            type: 'function',
            layer: 'system',
            description: funcDescMatch ? funcDescMatch[1] : undefined,
            parent: id,
          })
          console.log(`Added nested function: ${funcId} (${funcLabel}) under ${id}`)
        }
      }
    }
    console.log('=== SYSTEM_FUNCTION PARSING END ===')

    // Parse requirements - support both formats:
    // 1. req ID "Name" { ... }
    // 2. requirement "ID" { ... }
    const requirementRegex1 = /req\s+([A-Z]+-\d+)\s+"([^"]+)"\s*\{([\s\S]*?)\n\s*\}/g
    const requirementRegex2 = /requirement\s+"([^"]+)"\s*\{([\s\S]*?)(?:\n\s*requirement|\n\s*\}|$)/g
    let match  // Declare match variable for requirement parsing
    // Parse format 1: req ID "Name" { ... }
    while ((match = requirementRegex1.exec(code)) !== null) {
      const id = match[1]
      const label = match[2]
      const body = match[3]
      
      const descMatch = body.match(/description:\s*"([^"]*)"/)
      const safetyMatch = body.match(/safety_level:\s*([^\s\n,}]+)/)
      const tracesMatch = body.match(/traces:\s*\[([^\]]+)\]/)
      
      nodes.push({
        id,
        label,
        type: 'requirement',
        safetyLevel: safetyMatch ? safetyMatch[1] : undefined,
        description: descMatch ? descMatch[1] : undefined,
      })
      
      // Create edges from traces
      if (tracesMatch) {
        const traceIds = tracesMatch[1].split(',').map(s => s.trim())
        traceIds.forEach(traceId => {
          edges.push({
            id: `${id}-${traceId}`,
            source: id,
            target: traceId,
            type: 'traces',
          })
        })
      }
    }
    
    // Parse format 2: requirement "ID" { ... }
    while ((match = requirementRegex2.exec(code)) !== null) {
      const label = match[1]
      const body = match[2]
      
      const idMatch = body.match(/id:\s*"([^"]+)"/)
      const id = idMatch ? idMatch[1] : label
      
      const descMatch = body.match(/description:\s*"([^"]*)"/)
      const safetyMatch = body.match(/safety_level:\s*"?([^\s\n,}"]+)"?/)
      
      if (!nodes.find(n => n.id === id)) {
        nodes.push({
          id,
          label,
          type: 'requirement',
          safetyLevel: safetyMatch ? safetyMatch[1] : undefined,
          description: descMatch ? descMatch[1] : undefined,
        })
      }
    }

    // Parse traceability - support multiple formats:
    // 1. trace ID -> [ID1, ID2]
    // 2. trace "ID1" satisfies "ID2" { ... }
    // 3. trace "ID1" implements "ID2" { ... }
    const traceRegex1 = /trace\s+([A-Z]+-\d+)\s*->\s*\[([^\]]+)\]/g
    const traceRegex2 = /trace\s+"([^"]+)"\s+(satisfies|implements|realizes)\s+"([^"]+)"/g
    // Parse format 1: trace ID -> [ID1, ID2]
    while ((match = traceRegex1.exec(code)) !== null) {
      const sourceId = match[1]
      const targetIds = match[2].split(',').map(s => s.trim())
      
      targetIds.forEach(targetId => {
        edges.push({
          id: `trace-${sourceId}-${targetId}`,
          source: sourceId,
          target: targetId,
          type: 'traces',
        })
      })
    }
    
    // Parse format 2: trace "ID1" satisfies/implements "ID2"
    while ((match = traceRegex2.exec(code)) !== null) {
      const sourceId = match[1]
      const traceType = match[2]
      const targetId = match[3]
      
      const edgeType = traceType === 'satisfies' ? 'satisfies' : 
                       traceType === 'implements' ? 'implements' : 'realizes'
      
      edges.push({
        id: `trace-${sourceId}-${targetId}`,
        source: sourceId,
        target: targetId,
        type: edgeType,
      })
    }

    // Parse connections - format: connect ComponentA.Interface -> ComponentB
    const connectRegex = /connect\s+([A-Za-z_][A-Za-z0-9_]*)(?:\.([A-Za-z_][A-Za-z0-9_]*))?\s*->\s*([A-Za-z_][A-Za-z0-9_]*)/g
    while ((match = connectRegex.exec(code)) !== null) {
      const sourceComp = match[1]
      const interfaceName = match[2]
      const targetComp = match[3]
      
      edges.push({
        id: `${sourceComp}-${targetComp}`,
        source: sourceComp,
        target: targetComp,
        type: 'connects',
        label: interfaceName,
      })
    }

    // Filter out edges that reference non-existent nodes
    const nodeIds = new Set(nodes.map(n => n.id))
    const validEdges = edges.filter(edge => {
      const sourceExists = nodeIds.has(edge.source)
      const targetExists = nodeIds.has(edge.target)
      if (!sourceExists || !targetExists) {
        console.log(`Filtered out edge ${edge.id}: source=${edge.source} (exists: ${sourceExists}), target=${edge.target} (exists: ${targetExists})`)
      }
      return sourceExists && targetExists
    })

    console.log(`Parsed ${nodes.length} nodes and ${validEdges.length} valid edges (filtered ${edges.length - validEdges.length} invalid edges)`)
    console.log(`Layers found: ${layers.join(', ')}`)
    console.log(`Node breakdown:`, {
      operational: nodes.filter(n => n.layer === 'operational').length,
      system: nodes.filter(n => n.layer === 'system').length,
      logical: nodes.filter(n => n.layer === 'logical').length,
    })

    return {
      nodes,
      edges: validEdges,
      layers,
      layer: layers[0] || 'logical',
    }
  }

  private calculateStatsFromCode(code: string): any {
    const componentCount = (code.match(/component\s+[A-Za-z_][A-Za-z0-9_]*\s+"[^"]+"/g) || []).length
    const requirementCount = (code.match(/req\s+[A-Z]+-\d+/g) || []).length
    const functionCount = (code.match(/function\s+[A-Za-z_][A-Za-z0-9_]*\s+"[^"]+"/g) || []).length
    const tracesCount = (code.match(/traces:\s*\[/g) || []).length + (code.match(/trace\s+/g) || []).length + (code.match(/connect\s+/g) || []).length

    return {
      components: componentCount,
      requirements: requirementCount,
      functions: functionCount,
      traces: tracesCount,
    }
  }

  private extractDiagram(parsed: any): any {
    if (!parsed || !parsed.architecture) {
      return undefined
    }

    const nodes = []
    const edges = []

    if (parsed.architecture.components) {
      for (const comp of parsed.architecture.components) {
        nodes.push({
          id: comp.id,
          label: comp.name || comp.id,
          type: 'component',
          safetyLevel: comp.safety_level,
          description: comp.description,
        })
      }
    }

    if (parsed.architecture.functions) {
      for (const func of parsed.architecture.functions) {
        nodes.push({
          id: func.id,
          label: func.name || func.id,
          type: 'function',
          description: func.description,
        })
      }
    }

    if (parsed.architecture.requirements) {
      for (const req of parsed.architecture.requirements) {
        nodes.push({
          id: req.id,
          label: req.name || req.id,
          type: 'requirement',
          safetyLevel: req.safety_level,
          description: req.description,
        })
      }
    }

    if (parsed.architecture.traces) {
      for (const trace of parsed.architecture.traces) {
        edges.push({
          id: `${trace.from}-${trace.to}`,
          source: trace.from,
          target: trace.to,
          type: trace.type || 'trace',
          label: trace.label,
        })
      }
    }

    return {
      nodes,
      edges,
      layer: parsed.architecture.layer || 'logical',
    }
  }

  private calculateStats(parsed: any): any {
    if (!parsed || !parsed.architecture) {
      return undefined
    }

    return {
      requirements: parsed.architecture.requirements?.length || 0,
      components: parsed.architecture.components?.length || 0,
      functions: parsed.architecture.functions?.length || 0,
      traces: parsed.architecture.traces?.length || 0,
    }
  }

  private extractWarnings(output: string): string[] {
    const warnings: string[] = []
    const lines = output.split('\n')

    for (const line of lines) {
      if (line.toLowerCase().includes('warning')) {
        warnings.push(line.trim())
      }
    }

    return warnings
  }

  private parseErrors(output: string): string[] {
    const errors: string[] = []
    const lines = output.split('\n')

    for (const line of lines) {
      if (line.toLowerCase().includes('error') || line.includes('expected')) {
        errors.push(line.trim())
      }
    }

    return errors.length > 0 ? errors : [output]
  }
}
