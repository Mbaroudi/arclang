import { PrismaClient } from '@prisma/client';
import crypto from 'crypto';

export interface LearningPattern {
  errorSignature: string;
  solutionPattern: string;
  frequency: number;
  successRate: number;
}

export class LearningService {
  private prisma: PrismaClient;

  constructor(prisma: PrismaClient) {
    this.prisma = prisma;
  }

  async recordCorrection(correction: any, conversation: any): Promise<void> {
    const errorSignature = this.generateErrorSignature(correction);
    const solutionPattern = this.extractSolutionPattern(correction);

    const userId = conversation.userId;
    const diagramType = conversation.context?.diagramType;

    await this.prisma.errorPattern.upsert({
      where: {
        errorSignature_userId: {
          errorSignature,
          userId: userId || null,
        },
      },
      create: {
        userId: userId || null,
        errorSignature,
        solutionPattern,
        diagramType,
        frequency: 1,
        successRate: 0.0,
      },
      update: {
        frequency: {
          increment: 1,
        },
        solutionPattern,
        lastSeen: new Date(),
      },
    });

    await this.prisma.auditLog.create({
      data: {
        action: 'CORRECTION_RECORDED',
        entityType: 'ErrorPattern',
        entityId: errorSignature,
        userId,
        metadata: {
          issueType: correction.issueType,
          diagramType,
          resolved: correction.resolved,
        },
      },
    });
  }

  async recordSuccess(correction: any): Promise<void> {
    const errorSignature = this.generateErrorSignature(correction);

    const pattern = await this.prisma.errorPattern.findFirst({
      where: { errorSignature },
    });

    if (pattern) {
      const newSuccessRate = this.calculateNewSuccessRate(
        pattern.successRate,
        pattern.frequency,
        true
      );

      await this.prisma.errorPattern.update({
        where: { id: pattern.id },
        data: { successRate: newSuccessRate },
      });
    }
  }

  async recordFailure(correction: any): Promise<void> {
    const errorSignature = this.generateErrorSignature(correction);

    const pattern = await this.prisma.errorPattern.findFirst({
      where: { errorSignature },
    });

    if (pattern) {
      const newSuccessRate = this.calculateNewSuccessRate(
        pattern.successRate,
        pattern.frequency,
        false
      );

      await this.prisma.errorPattern.update({
        where: { id: pattern.id },
        data: { successRate: newSuccessRate },
      });
    }
  }

  async getTopErrorPatterns(
    limit: number = 10,
    userId?: string,
    diagramType?: string
  ): Promise<LearningPattern[]> {
    const patterns = await this.prisma.errorPattern.findMany({
      where: {
        OR: [{ userId }, { userId: null }],
        ...(diagramType && { diagramType }),
      },
      orderBy: [{ frequency: 'desc' }, { successRate: 'asc' }],
      take: limit,
    });

    return patterns.map((p) => ({
      errorSignature: p.errorSignature,
      solutionPattern: p.solutionPattern,
      frequency: p.frequency,
      successRate: p.successRate,
    }));
  }

  async analyzeUserProgress(userId: string, timeWindowDays: number = 30): Promise<any> {
    const since = new Date();
    since.setDate(since.getDate() - timeWindowDays);

    const corrections = await this.prisma.correction.findMany({
      where: {
        conversation: { userId },
        createdAt: { gte: since },
      },
      include: {
        conversation: true,
      },
    });

    const totalCorrections = corrections.length;
    const resolvedCorrections = corrections.filter((c) => c.resolved).length;
    const successRate = totalCorrections > 0 ? resolvedCorrections / totalCorrections : 0;

    const issueTypes = corrections.reduce((acc, c) => {
      acc[c.issueType] = (acc[c.issueType] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    return {
      totalCorrections,
      resolvedCorrections,
      successRate,
      issueTypes,
      timeWindowDays,
    };
  }

  private generateErrorSignature(correction: any): string {
    const content = `${correction.issueType}:${correction.userFeedback}`;
    return crypto.createHash('sha256').update(content).digest('hex').substring(0, 16);
  }

  private extractSolutionPattern(correction: any): string {
    return `${correction.issueType}: ${correction.userFeedback.substring(0, 200)}`;
  }

  private calculateNewSuccessRate(
    currentRate: number,
    totalAttempts: number,
    success: boolean
  ): number {
    const currentSuccesses = currentRate * totalAttempts;
    const newSuccesses = currentSuccesses + (success ? 1 : 0);
    const newTotal = totalAttempts + 1;
    return newSuccesses / newTotal;
  }
}

export function createLearningService(prisma: PrismaClient): LearningService {
  return new LearningService(prisma);
}
