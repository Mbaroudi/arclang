import { FastifyPluginAsync } from 'fastify'
import { createFileSchema, updateFileSchema } from '../types'

export const fileRoutes: FastifyPluginAsync = async (fastify) => {
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

      const files = await fastify.prisma.file.findMany({
        where: { projectId },
        include: {
          author: {
            select: {
              id: true,
              email: true,
              name: true,
            },
          },
        },
        orderBy: { updatedAt: 'desc' },
      })

      return reply.send({ files })
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

      const file = await fastify.prisma.file.findUnique({
        where: { id },
        include: {
          project: true,
          author: {
            select: {
              id: true,
              email: true,
              name: true,
            },
          },
        },
      })

      if (!file) {
        return reply.code(404).send({ error: 'File not found' })
      }

      if (file.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      return reply.send({ file })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })

  fastify.post('/', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const body = createFileSchema.parse(request.body)
      const { projectId } = request.query as { projectId: string }

      if (!projectId) {
        return reply.code(400).send({ error: 'projectId is required' })
      }

      const project = await fastify.prisma.project.findFirst({
        where: {
          id: projectId,
          ownerId: request.user!.id,
        },
      })

      if (!project) {
        return reply.code(404).send({ error: 'Project not found' })
      }

      const file = await fastify.prisma.file.create({
        data: {
          name: body.name,
          path: body.path,
          content: body.content,
          language: body.language,
          projectId,
          authorId: request.user!.id,
        },
        include: {
          author: {
            select: {
              id: true,
              email: true,
              name: true,
            },
          },
        },
      })

      await fastify.prisma.auditLog.create({
        data: {
          action: 'CREATE_FILE',
          entityType: 'FILE',
          entityId: file.id,
          userId: request.user!.id,
          metadata: { name: file.name, projectId },
        },
      })

      return reply.code(201).send({ file })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Invalid request' })
    }
  })

  fastify.patch('/:id', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { id } = request.params as { id: string }
      const body = updateFileSchema.parse(request.body)

      const existingFile = await fastify.prisma.file.findUnique({
        where: { id },
        include: { project: true },
      })

      if (!existingFile) {
        return reply.code(404).send({ error: 'File not found' })
      }

      if (existingFile.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      const file = await fastify.prisma.file.update({
        where: { id },
        data: body,
        include: {
          author: {
            select: {
              id: true,
              email: true,
              name: true,
            },
          },
        },
      })

      await fastify.prisma.auditLog.create({
        data: {
          action: 'UPDATE_FILE',
          entityType: 'FILE',
          entityId: file.id,
          userId: request.user!.id,
          metadata: body,
        },
      })

      return reply.send({ file })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Invalid request' })
    }
  })

  fastify.delete('/:id', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const { id } = request.params as { id: string }

      const existingFile = await fastify.prisma.file.findUnique({
        where: { id },
        include: { project: true },
      })

      if (!existingFile) {
        return reply.code(404).send({ error: 'File not found' })
      }

      if (existingFile.project.ownerId !== request.user!.id) {
        return reply.code(403).send({ error: 'Access denied' })
      }

      await fastify.prisma.file.delete({
        where: { id },
      })

      await fastify.prisma.auditLog.create({
        data: {
          action: 'DELETE_FILE',
          entityType: 'FILE',
          entityId: id,
          userId: request.user!.id,
          metadata: { name: existingFile.name },
        },
      })

      return reply.send({ message: 'File deleted successfully' })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })
}
