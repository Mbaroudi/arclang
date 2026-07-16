import { FastifyPluginAsync } from 'fastify';
import fp from 'fastify-plugin';
import { createConversationalAIService, ConversationalAIService } from '../services/conversational-ai';
import { createLearningService, LearningService } from '../services/learning';

declare module 'fastify' {
  interface FastifyInstance {
    aiService: ConversationalAIService;
    learningService: LearningService;
  }
}

const servicesPlugin: FastifyPluginAsync = async (fastify) => {
  const aiService = createConversationalAIService(fastify.prisma);
  const learningService = createLearningService(fastify.prisma);

  fastify.decorate('aiService', aiService);
  fastify.decorate('learningService', learningService);

  fastify.log.info('Services plugin registered');
};

export default fp(servicesPlugin);
