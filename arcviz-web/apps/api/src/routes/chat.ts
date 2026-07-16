import { FastifyInstance, FastifyRequest, FastifyReply } from 'fastify';
import { z } from 'zod';

const createConversationSchema = z.object({
  projectId: z.string().optional().nullable(),
  initialMessage: z.string().optional(),
  context: z.record(z.any()).optional(),
});

const sendMessageSchema = z.object({
  content: z.string().min(1),
  context: z.record(z.any()).optional(),
});

const submitFeedbackSchema = z.object({
  rating: z.number().min(1).max(5),
  helpful: z.boolean().optional(),
  accurate: z.boolean().optional(),
  comment: z.string().optional(),
});

const submitCorrectionSchema = z.object({
  originalCode: z.string(),
  correctedCode: z.string(),
  userFeedback: z.string(),
  issueType: z.enum(['syntax', 'semantic', 'missing', 'wrong', 'other']),
  issueDetails: z.record(z.any()).optional(),
});

export default async function chatRoutes(fastify: FastifyInstance) {
  fastify.post(
    '/conversations',
    async (
      request: FastifyRequest<{
        Body: z.infer<typeof createConversationSchema>;
      }>,
      reply: FastifyReply
    ) => {
      const body = createConversationSchema.parse(request.body);
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const conversation = await fastify.prisma.conversation.create({
        data: {
          userId,
          projectId: body.projectId || null,
          title: body.initialMessage
            ? body.initialMessage.substring(0, 100)
            : 'New Conversation',
          context: body.context || {},
        },
        include: {
          messages: true,
        },
      });

      if (body.initialMessage) {
        await fastify.prisma.message.create({
          data: {
            conversationId: conversation.id,
            role: 'USER',
            content: body.initialMessage,
          },
        });
      }

      const conversationWithMessages = await fastify.prisma.conversation.findUnique({
        where: { id: conversation.id },
        include: {
          messages: {
            orderBy: { createdAt: 'asc' },
            include: { feedback: true },
          },
        },
      });

      return reply.send(conversationWithMessages);
    }
  );

  fastify.get(
    '/conversations/:id',
    async (
      request: FastifyRequest<{
        Params: { id: string };
      }>,
      reply: FastifyReply
    ) => {
      const { id } = request.params;
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const conversation = await fastify.prisma.conversation.findFirst({
        where: {
          id,
          userId,
        },
        include: {
          messages: {
            orderBy: { createdAt: 'asc' },
            include: { feedback: true },
          },
        },
      });

      if (!conversation) {
        return reply.code(404).send({ error: 'Conversation not found' });
      }

      return reply.send(conversation);
    }
  );

  fastify.get(
    '/conversations',
    async (
      request: FastifyRequest<{
        Querystring: { projectId?: string };
      }>,
      reply: FastifyReply
    ) => {
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';
      const { projectId } = request.query;

      const conversations = await fastify.prisma.conversation.findMany({
        where: {
          userId,
          ...(projectId && { projectId }),
        },
        include: {
          messages: {
            take: 1,
            orderBy: { createdAt: 'desc' },
          },
          _count: {
            select: { messages: true },
          },
        },
        orderBy: { updatedAt: 'desc' },
      });

      return reply.send(conversations);
    }
  );

  fastify.post(
    '/conversations/:id/messages',
    async (
      request: FastifyRequest<{
        Params: { id: string };
        Body: z.infer<typeof sendMessageSchema>;
      }>,
      reply: FastifyReply
    ) => {
      const { id } = request.params;
      const body = sendMessageSchema.parse(request.body);
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const conversation = await fastify.prisma.conversation.findFirst({
        where: { id, userId },
        include: {
          messages: {
            orderBy: { createdAt: 'asc' },
          },
        },
      });

      if (!conversation) {
        return reply.code(404).send({ error: 'Conversation not found' });
      }

      const userMessage = await fastify.prisma.message.create({
        data: {
          conversationId: id,
          role: 'USER',
          content: body.content,
        },
      });

      const conversationHistory = conversation.messages.map((msg) => ({
        role: msg.role.toLowerCase(),
        content: msg.content,
      }));

      conversationHistory.push({
        role: 'user',
        content: body.content,
      });

      const aiService = fastify.aiService;
      const contextData = {
        ...(typeof conversation.context === 'object' && conversation.context !== null ? conversation.context : {}),
        ...(body.context || {}),
      };
      
      let aiResponse;
      try {
        aiResponse = await aiService.generateResponse(
          conversationHistory,
          contextData
        );
      } catch (error: any) {
        fastify.log.error('AI Service error:', error);
        return reply.code(500).send({ 
          error: 'AI service failed', 
          details: error.message 
        });
      }

      const assistantMessage = await fastify.prisma.message.create({
        data: {
          conversationId: id,
          role: 'ASSISTANT',
          content: aiResponse.content || 'No response generated',
          generatedCode: aiResponse.generatedCode,
          diagramSvg: aiResponse.diagramSvg,
          diagramType: aiResponse.diagramType,
          actions: aiResponse.actions || [],
        },
      });

      await fastify.prisma.conversation.update({
        where: { id },
        data: { updatedAt: new Date() },
      });

      return reply.send({
        userMessage,
        assistantMessage,
      });
    }
  );

  fastify.post(
    '/messages/:id/feedback',
    async (
      request: FastifyRequest<{
        Params: { id: string };
        Body: z.infer<typeof submitFeedbackSchema>;
      }>,
      reply: FastifyReply
    ) => {
      const { id } = request.params;
      const body = submitFeedbackSchema.parse(request.body);
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const message = await fastify.prisma.message.findFirst({
        where: {
          id,
          conversation: { userId },
        },
      });

      if (!message) {
        return reply.code(404).send({ error: 'Message not found' });
      }

      const feedback = await fastify.prisma.feedback.upsert({
        where: { messageId: id },
        create: {
          messageId: id,
          rating: body.rating,
          helpful: body.helpful,
          accurate: body.accurate,
          comment: body.comment,
        },
        update: {
          rating: body.rating,
          helpful: body.helpful,
          accurate: body.accurate,
          comment: body.comment,
        },
      });

      return reply.send(feedback);
    }
  );

  fastify.post(
    '/messages/:id/correct',
    async (
      request: FastifyRequest<{
        Params: { id: string };
        Body: z.infer<typeof submitCorrectionSchema>;
      }>,
      reply: FastifyReply
    ) => {
      const { id } = request.params;
      const body = submitCorrectionSchema.parse(request.body);
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const message = await fastify.prisma.message.findFirst({
        where: {
          id,
          conversation: { userId },
        },
        include: { conversation: true },
      });

      if (!message) {
        return reply.code(404).send({ error: 'Message not found' });
      }

      const correction = await fastify.prisma.correction.create({
        data: {
          conversationId: message.conversationId,
          originalMessageId: id,
          originalCode: body.originalCode,
          correctedCode: body.correctedCode,
          userFeedback: body.userFeedback,
          issueType: body.issueType,
          issueDetails: body.issueDetails,
        },
      });

      const aiService = fastify.aiService;
      const conversationHistory = await fastify.prisma.message.findMany({
        where: { conversationId: message.conversationId },
        orderBy: { createdAt: 'asc' },
      });

      const history = conversationHistory.map((msg) => ({
        role: msg.role.toLowerCase(),
        content: msg.content,
      }));

      history.push({
        role: 'user',
        content: `I found an issue with your previous response. ${body.userFeedback}\n\nOriginal code:\n${body.originalCode}\n\nCorrected code:\n${body.correctedCode}\n\nPlease regenerate based on this correction.`,
      });

      const aiResponse = await aiService.generateResponse(history, {
        issueType: body.issueType,
      } as any);

      const correctedMessage = await fastify.prisma.message.create({
        data: {
          conversationId: message.conversationId,
          role: 'ASSISTANT',
          content: aiResponse.content,
          generatedCode: aiResponse.generatedCode,
          diagramSvg: aiResponse.diagramSvg,
          diagramType: aiResponse.diagramType,
        },
      });

      await fastify.prisma.correction.update({
        where: { id: correction.id },
        data: {
          resolved: true,
          resolutionAttempts: 1,
        },
      });

      await fastify.learningService.recordCorrection(correction, message.conversation);

      return reply.send({
        correction,
        correctedMessage,
      });
    }
  );

  fastify.delete(
    '/conversations/:id',
    async (
      request: FastifyRequest<{
        Params: { id: string };
      }>,
      reply: FastifyReply
    ) => {
      const { id } = request.params;
      const userId = (request as any).user?.id || 'cmhbzei430000pdhh58zdw8kc';

      const conversation = await fastify.prisma.conversation.findFirst({
        where: { id, userId },
      });

      if (!conversation) {
        return reply.code(404).send({ error: 'Conversation not found' });
      }

      await fastify.prisma.conversation.delete({
        where: { id },
      });

      return reply.send({ success: true });
    }
  );
}
