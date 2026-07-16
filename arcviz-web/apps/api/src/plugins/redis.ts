import { FastifyPluginAsync } from 'fastify'
import fp from 'fastify-plugin'
import Redis from 'ioredis'

declare module 'fastify' {
  interface FastifyInstance {
    redis: Redis
  }
}

const redisPlugin: FastifyPluginAsync = async (fastify) => {
  const redis = new Redis({
    host: process.env.REDIS_HOST || 'localhost',
    port: parseInt(process.env.REDIS_PORT || '6379', 10),
    password: process.env.REDIS_PASSWORD,
    retryStrategy: (times) => {
      const delay = Math.min(times * 50, 2000)
      return delay
    },
  })

  redis.on('connect', () => {
    fastify.log.info('Redis connected')
  })

  redis.on('error', (err) => {
    fastify.log.error({ err }, 'Redis connection error')
  })

  fastify.decorate('redis', redis)

  fastify.addHook('onClose', async (instance) => {
    await instance.redis.quit()
  })
}

export default fp(redisPlugin)
export { redisPlugin }
