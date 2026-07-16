import { FastifyPluginAsync } from 'fastify'
import { registerSchema, loginSchema } from '../types'
import { hashPassword, verifyPassword } from '../utils/password'

export const authRoutes: FastifyPluginAsync = async (fastify) => {
  fastify.post('/register', async (request, reply) => {
    try {
      const body = registerSchema.parse(request.body)

      const existingUser = await fastify.prisma.user.findUnique({
        where: { email: body.email },
      })

      if (existingUser) {
        return reply.code(400).send({ error: 'User already exists' })
      }

      const passwordHash = await hashPassword(body.password)

      const user = await fastify.prisma.user.create({
        data: {
          email: body.email,
          passwordHash,
          name: body.name,
        },
        select: {
          id: true,
          email: true,
          name: true,
          role: true,
          createdAt: true,
        },
      })

      const token = fastify.jwt.sign({
        id: user.id,
        email: user.email,
        role: user.role,
      })

      const refreshToken = fastify.jwt.sign(
        { id: user.id },
        { expiresIn: '30d' }
      )

      await fastify.prisma.session.create({
        data: {
          userId: user.id,
          token,
          refreshToken,
          expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
        },
      })

      return reply.send({
        user,
        token,
        refreshToken,
      })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Invalid request' })
    }
  })

  fastify.post('/login', async (request, reply) => {
    try {
      const body = loginSchema.parse(request.body)

      const user = await fastify.prisma.user.findUnique({
        where: { email: body.email },
      })

      if (!user) {
        return reply.code(401).send({ error: 'Invalid credentials' })
      }

      const isValid = await verifyPassword(body.password, user.passwordHash)

      if (!isValid) {
        return reply.code(401).send({ error: 'Invalid credentials' })
      }

      const token = fastify.jwt.sign({
        id: user.id,
        email: user.email,
        role: user.role,
      })

      const refreshToken = fastify.jwt.sign(
        { id: user.id },
        { expiresIn: '30d' }
      )

      await fastify.prisma.session.create({
        data: {
          userId: user.id,
          token,
          refreshToken,
          expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
        },
      })

      await fastify.prisma.user.update({
        where: { id: user.id },
        data: { lastLoginAt: new Date() },
      })

      return reply.send({
        user: {
          id: user.id,
          email: user.email,
          name: user.name,
          role: user.role,
        },
        token,
        refreshToken,
      })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Invalid request' })
    }
  })

  fastify.post('/logout', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const token = request.headers.authorization?.replace('Bearer ', '')

      if (token) {
        await fastify.prisma.session.deleteMany({
          where: { token },
        })
      }

      return reply.send({ message: 'Logged out successfully' })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(400).send({ error: 'Logout failed' })
    }
  })

  fastify.get('/me', {
    preHandler: fastify.authenticate,
  }, async (request, reply) => {
    try {
      const user = await fastify.prisma.user.findUnique({
        where: { id: request.user!.id },
        select: {
          id: true,
          email: true,
          name: true,
          role: true,
          avatarUrl: true,
          createdAt: true,
          updatedAt: true,
        },
      })

      if (!user) {
        return reply.code(404).send({ error: 'User not found' })
      }

      return reply.send({ user })
    } catch (error) {
      fastify.log.error(error)
      return reply.code(500).send({ error: 'Internal server error' })
    }
  })
}
