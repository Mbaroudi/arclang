import Fastify from 'fastify'
import cors from '@fastify/cors'
import multipart from '@fastify/multipart'
import jwt from '@fastify/jwt'
import cookie from '@fastify/cookie'
import dotenv from 'dotenv'

import { prismaPlugin } from './plugins/prisma'
import { redisPlugin } from './plugins/redis'
import { authPlugin } from './plugins/auth'
import servicesPlugin from './plugins/services'

import { authRoutes } from './routes/auth'
import { projectRoutes } from './routes/projects'
import { fileRoutes } from './routes/files'
import { compilationRoutes } from './routes/compilation'
import { diagramRoutes } from './routes/diagrams'
import { aiRoutes } from './routes/ai'
import chatRoutes from './routes/chat'
import { arcadia7DRoutes } from './routes/arcadia-7d'

dotenv.config()

const PORT = parseInt(process.env.PORT || '4000', 10)
const HOST = process.env.HOST || '0.0.0.0'

const fastify = Fastify({
  logger: {
    level: process.env.LOG_LEVEL || 'info',
  },
})

async function start() {
  try {
    await fastify.register(cors, {
      origin: (origin, cb) => {
        const allowedOrigins = ['http://localhost:3002', 'null']
        if (!origin || allowedOrigins.includes(origin)) {
          cb(null, true)
        } else {
          cb(new Error('Not allowed by CORS'), false)
        }
      },
      credentials: true,
    })

    await fastify.register(multipart, {
      limits: {
        fileSize: 10 * 1024 * 1024,
      },
    })

    await fastify.register(cookie)

    await fastify.register(jwt, {
      secret: process.env.JWT_SECRET || 'your-secret-key-change-in-production',
      sign: {
        expiresIn: '7d',
      },
    })

    await fastify.register(prismaPlugin)
    await fastify.register(redisPlugin)
    await fastify.register(authPlugin)
    await fastify.register(servicesPlugin)

    fastify.get('/health', async () => {
      return { status: 'ok', timestamp: new Date().toISOString() }
    })

    await fastify.register(authRoutes, { prefix: '/api/auth' })
    await fastify.register(projectRoutes, { prefix: '/api/projects' })
    await fastify.register(fileRoutes, { prefix: '/api/files' })
    await fastify.register(compilationRoutes, { prefix: '/api/compile' })
    await fastify.register(diagramRoutes, { prefix: '/api/diagrams' })
    await fastify.register(aiRoutes, { prefix: '/api/ai' })
    await fastify.register(chatRoutes, { prefix: '/api/chat' })
    await fastify.register(arcadia7DRoutes, { prefix: '/api/arcadia-7d' })

    await fastify.listen({ port: PORT, host: HOST })

    console.log(`🚀 ArcViz API Server running at http://${HOST}:${PORT}`)
    console.log(`📊 Health check: http://${HOST}:${PORT}/health`)
  } catch (err) {
    fastify.log.error(err)
    process.exit(1)
  }
}

start()
