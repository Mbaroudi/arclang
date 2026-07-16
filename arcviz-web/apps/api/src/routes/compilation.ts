import { FastifyPluginAsync } from 'fastify'
import { compileSchema } from '../types'
import { ArcLangCompiler } from '../services/compiler'

export const compilationRoutes: FastifyPluginAsync = async (fastify) => {
  const compiler = new ArcLangCompiler()

  fastify.post('/', async (request, reply) => {
    // Allow compilation without auth for now (development mode)
    const userId = request.user?.id || null
    try {
      const body = compileSchema.parse(request.body)

      const result = await compiler.compile(body.code)

      let compilationRecord
      if (body.fileId && userId) {
        const file = await fastify.prisma.file.findUnique({
          where: { id: body.fileId },
          include: { project: true },
        })

        if (!file) {
          return reply.code(404).send({ error: 'File not found' })
        }

        if (file.project.ownerId !== userId) {
          return reply.code(403).send({ error: 'Access denied' })
        }

        compilationRecord = await fastify.prisma.compilation.create({
          data: {
            success: result.success,
            output: result.output,
            errors: result.errors,
            warnings: result.warnings,
            stats: result.stats as any,
            fileId: body.fileId,
          },
        })

        if (result.success && result.diagram) {
          await fastify.prisma.diagram.create({
            data: {
              name: `${file.name} - ${new Date().toISOString()}`,
              layer: result.diagram.layer.toUpperCase() as any,
              nodes: result.diagram.nodes as any,
              edges: result.diagram.edges as any,
              projectId: file.projectId,
              compilationId: compilationRecord.id,
            },
          })
        }

        if (userId) {
          await fastify.prisma.auditLog.create({
            data: {
              action: 'COMPILE_FILE',
              entityType: 'FILE',
              entityId: body.fileId,
              userId: userId,
              metadata: {
                success: result.success,
                stats: result.stats,
              },
            },
          })
        }
      }

      const response = {
        ...result,
        compilationId: compilationRecord?.id,
      }
      
      fastify.log.info(`Compilation response: success=${response.success}, nodes=${response.diagram?.nodes?.length || 0}, edges=${response.diagram?.edges?.length || 0}`)
      
      return reply.send(response)
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Compilation failed' })
    }
  })

  fastify.post('/validate', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { code } = request.body as { code: string }

      if (!code) {
        return reply.code(400).send({ error: 'Code is required' })
      }

      const result = await compiler.validate(code)

      return reply.send(result)
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Validation failed' })
    }
  })

  fastify.get('/history/:fileId', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { fileId } = request.params as { fileId: string }

      const file = await fastify.prisma.file.findUnique({
        where: { id: fileId },
        include: { project: true },
      })

      if (!file) {
        return reply.code(404).send({ error: 'File not found' })
      }

      if (file.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      const compilations = await fastify.prisma.compilation.findMany({
        where: { fileId },
        orderBy: { createdAt: 'desc' },
        take: 50,
      })

      return reply.send({ compilations })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })
}
