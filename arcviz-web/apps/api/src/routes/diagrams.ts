import { FastifyPluginAsync } from 'fastify'
import { generateDiagram, type DiagramType } from '../services/diagram-generator'
import { getSampleModel } from '../services/sample-models'
import { ArcLangParser } from '../services/arclang-parser'

export const diagramRoutes: FastifyPluginAsync = async (fastify) => {
  fastify.get('/project/:projectId', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { projectId } = request.params as { projectId: string }

      const project = await fastify.prisma.project.findFirst({
        where: {
          id: projectId,
          ownerId: request.user!.id,
        },
      })

      if (!project) {
        return reply.code(404).send({ error: 'Project not found' })
      }

      const diagrams = await fastify.prisma.diagram.findMany({
        where: { projectId },
        include: {
          compilation: {
            select: {
              success: true,
              createdAt: true,
            },
          },
        },
        orderBy: { updatedAt: 'desc' },
      })

      return reply.send({ diagrams })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })

  fastify.get('/:id', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { id } = request.params as { id: string }

      const diagram = await fastify.prisma.diagram.findUnique({
        where: { id },
        include: {
          project: true,
          compilation: true,
        },
      })

      if (!diagram) {
        return reply.code(404).send({ error: 'Diagram not found' })
      }

      if (diagram.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      return reply.send({ diagram })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })

  fastify.delete('/:id', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { id } = request.params as { id: string }

      const diagram = await fastify.prisma.diagram.findUnique({
        where: { id },
        include: { project: true },
      })

      if (!diagram) {
        return reply.code(404).send({ error: 'Diagram not found' })
      }

      if (diagram.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      await fastify.prisma.diagram.delete({
        where: { id },
      })

      await fastify.prisma.auditLog.create({
        data: {
          action: 'DELETE_DIAGRAM',
          entityType: 'DIAGRAM',
          entityId: id,
          userId: request.user!.id,
          metadata: { name: diagram.name },
        },
      })

      return reply.send({ message: 'Diagram deleted successfully' })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })

  fastify.post('/generate', async (request, reply) => {
    try {
      const { diagramType, outputPath, code } = request.body as any

      const validTypes = [
        'operational', 'functional', 'component', 'sequence',
        'state-machine', 'physical', 'class', 'tree',
        'capability', 'functional-chain'
      ]

      if (!validTypes.includes(diagramType)) {
        return reply.code(400).send({
          error: 'Invalid diagram type',
          validTypes
        })
      }

      let modelData
      let usedSampleData = false
      
      if (code && code.trim().length > 50) {
        const parser = new ArcLangParser()
        modelData = await parser.parseCode(code, diagramType)
        
        if (modelData) {
          fastify.log.info(`Using parsed code for ${diagramType}`)
          usedSampleData = false
        } else {
          fastify.log.warn(`Parser returned null for ${diagramType}`)
          modelData = getSampleModel(diagramType)
          usedSampleData = true
        }
      } else {
        fastify.log.info(`No code provided, using sample data for ${diagramType}`)
        modelData = getSampleModel(diagramType)
        usedSampleData = true
      }

      if (!modelData) {
        return reply.code(400).send({
          error: 'No model data available for this diagram type'
        })
      }

      const result = await generateDiagram(diagramType as DiagramType, modelData)
      
      if (!result.success) {
        return reply.code(500).send({
          error: 'Diagram generation failed',
          message: result.error
        })
      }

      return reply.send({
        success: true,
        diagramType: result.diagramType,
        svg: result.svg,
        outputPath: outputPath || `${diagramType}.svg`,
        size: result.size,
        elementCount: result.elementCount,
        features: result.features,
        usedSampleData
      })
    } catch (error: any) {
      fastify.log.error(error)
      return reply.code(500).send({
        error: 'Diagram generation failed',
        message: error.message
      })
    }
  })

  fastify.post('/generate-all', async (request, reply) => {
    try {
      const { outputDir, code } = request.body as any

      const diagramTypes: DiagramType[] = [
        'operational', 'functional', 'component', 'sequence',
        'state-machine', 'physical', 'class', 'tree',
        'capability', 'functional-chain'
      ]

      const results = await Promise.allSettled(
        diagramTypes.map(async (type) => {
          let modelData
          let usedSampleData = false
          
          if (code && code.trim().length > 50) {
            const parser = new ArcLangParser()
            modelData = await parser.parseCode(code, type)
            
            if (!modelData) {
              fastify.log.warn(`Parser returned null for ${type}, using sample data`)
              modelData = getSampleModel(type)
              usedSampleData = true
            } else {
              fastify.log.info(`Using parsed code for ${type}`)
              usedSampleData = false
            }
          } else {
            fastify.log.info(`No code provided for ${type}, using sample data`)
            modelData = getSampleModel(type)
            usedSampleData = true
          }
          
          if (!modelData) {
            throw new Error(`No model data for ${type}`)
          }
          const result = await generateDiagram(type, modelData)
          if (!result.success) {
            throw new Error(result.error || 'Generation failed')
          }
          return {
            type,
            svg: result.svg,
            outputPath: `${outputDir || './diagrams'}/${type}.svg`,
            size: result.size,
            elementCount: result.elementCount,
            features: result.features,
            usedSampleData
          }
        })
      )

      const successful = results.filter(r => r.status === 'fulfilled')
      const failed = results.filter(r => r.status === 'rejected')

      return reply.send({
        success: successful.length > 0,
        total: 10,
        successful: successful.length,
        failed: failed.length,
        diagrams: successful.map(r => (r as any).value),
        errors: failed.map(r => (r as any).reason?.message)
      })
    } catch (error: any) {
      fastify.log.error(error)
      return reply.code(500).send({
        error: 'Bulk diagram generation failed',
        message: error.message
      })
    }
  })

  fastify.get('/types', async (_request, reply) => {
    return reply.send({
      types: [
        { id: 'operational', name: 'Operational Activity', description: 'Swimlane activity diagrams' },
        { id: 'functional', name: 'Functional Dataflow', description: 'Function and data flow' },
        { id: 'component', name: 'Component Architecture', description: 'Block diagrams' },
        { id: 'sequence', name: 'Sequence Diagram', description: 'Interaction scenarios' },
        { id: 'state-machine', name: 'State Machine', description: 'State transitions' },
        { id: 'physical', name: 'Physical Architecture', description: 'Hardware deployment' },
        { id: 'class', name: 'Class Diagram', description: 'Data types and classes' },
        { id: 'tree', name: 'Tree Diagram', description: 'Hierarchical breakdown' },
        { id: 'capability', name: 'Capability Diagram', description: 'Requirements hierarchy' },
        { id: 'functional-chain', name: 'Functional Chain', description: 'Execution scenarios' }
      ]
    })
  })
}
