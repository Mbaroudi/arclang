import { FastifyPluginAsync, FastifyRequest } from 'fastify'
import fp from 'fastify-plugin'

declare module 'fastify' {
  interface FastifyRequest {
    user?: {
      id: string
      email: string
      role: string
    }
  }
}

const authPlugin: FastifyPluginAsync = async (fastify) => {
  fastify.decorate('authenticate', async (request: FastifyRequest) => {
    try {
      const token = request.headers.authorization?.replace('Bearer ', '')
      
      if (!token) {
        throw new Error('No token provided')
      }

      const decoded = fastify.jwt.verify(token) as {
        id: string
        email: string
        role: string
      }

      const session = await fastify.prisma.session.findUnique({
        where: { token },
        include: { user: true },
      })

      if (!session || session.expiresAt < new Date()) {
        throw new Error('Invalid or expired token')
      }

      request.user = {
        id: decoded.id,
        email: decoded.email,
        role: decoded.role,
      }
    } catch (err) {
      throw new Error('Authentication failed')
    }
  })
}

export default fp(authPlugin)
export { authPlugin }
