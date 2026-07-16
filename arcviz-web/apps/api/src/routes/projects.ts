import { FastifyPluginAsync } from 'fastify'
import { createProjectSchema, updateProjectSchema } from '../types'

export const projectRoutes: FastifyPluginAsync = async (fastify) => {
  fastify.get('/', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const projects = await fastify.prisma.project.findMany({
        where: { ownerId: request.user!.id },
        include: {
          _count: {
            select: { files: true, diagrams: true },
          },
        },
        orderBy: { updatedAt: 'desc' },
      })

      return reply.send({ projects })
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

      const project = await fastify.prisma.project.findFirst({
        where: {
          id,
          ownerId: request.user!.id,
        },
        include: {
          files: {
            orderBy: { updatedAt: 'desc' },
          },
          diagrams: {
            orderBy: { updatedAt: 'desc' },
          },
          owner: {
            select: {
              id: true,
              email: true,
              name: true,
            },
          },
        },
      })

      if (!project) {
        return reply.code(404).send({ error: 'Project not found' })
      }

      return reply.send({ project })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })

  fastify.post('/', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const body = createProjectSchema.parse(request.body)

      const project = await fastify.prisma.project.create({
        data: {
          name: body.name,
          description: body.description,
          visibility: body.visibility,
          ownerId: request.user!.id,
        },
        include: {
          owner: {
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
          action: 'CREATE_PROJECT',
          entityType: 'PROJECT',
          entityId: project.id,
          userId: request.user!.id,
          metadata: { name: project.name },
        },
      })

      return reply.code(201).send({ project })
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
      const body = updateProjectSchema.parse(request.body)

      const existingProject = await fastify.prisma.project.findFirst({
        where: {
          id,
          ownerId: request.user!.id,
        },
      })

      if (!existingProject) {
        return reply.code(404).send({ error: 'Project not found' })
      }

      const project = await fastify.prisma.project.update({
        where: { id },
        data: body,
        include: {
          owner: {
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
          action: 'UPDATE_PROJECT',
          entityType: 'PROJECT',
          entityId: project.id,
          userId: request.user!.id,
          metadata: body,
        },
      })

      return reply.send({ project })
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

      const existingProject = await fastify.prisma.project.findFirst({
        where: {
          id,
          ownerId: request.user!.id,
        },
      })

      if (!existingProject) {
        return reply.code(404).send({ error: 'Project not found' })
      }

      await fastify.prisma.project.delete({
        where: { id },
      })

      await fastify.prisma.auditLog.create({
        data: {
          action: 'DELETE_PROJECT',
          entityType: 'PROJECT',
          entityId: id,
          userId: request.user!.id,
          metadata: { name: existingProject.name },
        },
      })

      return reply.send({ message: 'Project deleted successfully' })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })
}
