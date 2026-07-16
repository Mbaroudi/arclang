import { FastifyPluginAsync } from 'fastify'
import { parseArcLangCode } from '../services/arcadia-7d-parser'

export const arcadia7DRoutes: FastifyPluginAsync = async (fastify) => {
  fastify.post('/parse', async (request, reply) => {
    try {
      const { code } = request.body as { code: string }
      
      if (!code || typeof code !== 'string') {
        return reply.code(400).send({
          success: false,
          error: 'Invalid code provided'
        })
      }
      
      const model = parseArcLangCode(code)
      
      const stats = {
        operational: {
          actors: model.operational.actors.length,
          activities: model.operational.activities.length,
          capabilities: model.operational.capabilities.length,
          interactions: model.operational.interactions.length
        },
        system: {
          actors: model.system.actors.length,
          functions: model.system.functions.length,
          interactions: model.system.interactions.length
        },
        logical: {
          components: model.logical.components.length,
          interfaces: model.logical.interfaces.length,
          dataFlows: model.logical.dataFlows.length
        },
        physical: {
          nodes: model.physical.nodes.length,
          links: model.physical.links.length,
          deployments: model.physical.deployments.length
        },
        epbs: {
          subsystems: model.epbs.subsystems.length,
          assemblies: model.epbs.assemblies.length,
          components: model.epbs.components.length
        },
        requirements: {
          requirements: model.requirements.requirements.length,
          traces: model.requirements.traces.length
        },
        crossCutting: {
          securityPolicies: model.crossCutting.securityPolicies.length,
          safetyConstraints: model.crossCutting.safetyConstraints.length,
          performanceMetrics: model.crossCutting.performanceMetrics.length
        }
      }
      
      fastify.log.info('Arcadia 7D model parsed successfully', stats)
      
      return reply.send({
        success: true,
        model,
        stats
      })
    } catch (error: any) {
      fastify.log.error('Failed to parse Arcadia 7D model:', error)
      return reply.code(500).send({
        success: false,
        error: error.message || 'Failed to parse code'
      })
    }
  })
  
  fastify.post('/validate', async (request, reply) => {
    try {
      const { code } = request.body as { code: string }
      
      if (!code || typeof code !== 'string') {
        return reply.code(400).send({
          success: false,
          error: 'Invalid code provided'
        })
      }
      
      const model = parseArcLangCode(code)
      
      const issues: Array<{ type: string; message: string; severity: 'error' | 'warning' | 'info' }> = []
      
      if (model.operational.actors.length === 0) {
        issues.push({
          type: 'operational',
          message: 'No actors defined in operational analysis',
          severity: 'warning'
        })
      }
      
      if (model.system.functions.length === 0) {
        issues.push({
          type: 'system',
          message: 'No system functions defined',
          severity: 'warning'
        })
      }
      
      if (model.logical.components.length === 0) {
        issues.push({
          type: 'logical',
          message: 'No components defined in logical architecture',
          severity: 'warning'
        })
      }
      
      if (model.physical.nodes.length === 0) {
        issues.push({
          type: 'physical',
          message: 'No physical nodes defined',
          severity: 'info'
        })
      }
      
      if (model.requirements.requirements.length === 0) {
        issues.push({
          type: 'requirements',
          message: 'No requirements defined',
          severity: 'warning'
        })
      }
      
      const hasErrors = issues.some(i => i.severity === 'error')
      
      return reply.send({
        success: !hasErrors,
        valid: !hasErrors,
        issues,
        summary: {
          errors: issues.filter(i => i.severity === 'error').length,
          warnings: issues.filter(i => i.severity === 'warning').length,
          info: issues.filter(i => i.severity === 'info').length
        }
      })
    } catch (error: any) {
      fastify.log.error('Failed to validate Arcadia 7D model:', error)
      return reply.code(500).send({
        success: false,
        error: error.message || 'Failed to validate code'
      })
    }
  })
}
