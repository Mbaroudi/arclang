-- CreateEnum
CREATE TYPE "MessageRole" AS ENUM ('USER', 'ASSISTANT', 'SYSTEM');

-- CreateTable
CREATE TABLE "conversations" (
    "id" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "projectId" TEXT,
    "title" TEXT,
    "context" JSONB,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "conversations_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "messages" (
    "id" TEXT NOT NULL,
    "conversationId" TEXT NOT NULL,
    "role" "MessageRole" NOT NULL,
    "content" TEXT NOT NULL,
    "generatedCode" TEXT,
    "diagramSvg" TEXT,
    "diagramType" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "messages_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "feedbacks" (
    "id" TEXT NOT NULL,
    "messageId" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "helpful" BOOLEAN,
    "accurate" BOOLEAN,
    "comment" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "feedbacks_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "corrections" (
    "id" TEXT NOT NULL,
    "conversationId" TEXT NOT NULL,
    "originalMessageId" TEXT NOT NULL,
    "originalCode" TEXT NOT NULL,
    "correctedCode" TEXT NOT NULL,
    "userFeedback" TEXT NOT NULL,
    "issueType" TEXT NOT NULL,
    "issueDetails" JSONB,
    "resolved" BOOLEAN NOT NULL DEFAULT false,
    "resolutionAttempts" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "corrections_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "error_patterns" (
    "id" TEXT NOT NULL,
    "userId" TEXT,
    "errorSignature" TEXT NOT NULL,
    "solutionPattern" TEXT NOT NULL,
    "diagramType" TEXT,
    "safetyLevel" TEXT,
    "frequency" INTEGER NOT NULL DEFAULT 1,
    "successRate" DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    "lastSeen" TIMESTAMP(3) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "error_patterns_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "conversations_userId_idx" ON "conversations"("userId");

-- CreateIndex
CREATE INDEX "conversations_projectId_idx" ON "conversations"("projectId");

-- CreateIndex
CREATE INDEX "messages_conversationId_idx" ON "messages"("conversationId");

-- CreateIndex
CREATE INDEX "messages_createdAt_idx" ON "messages"("createdAt");

-- CreateIndex
CREATE UNIQUE INDEX "feedbacks_messageId_key" ON "feedbacks"("messageId");

-- CreateIndex
CREATE INDEX "corrections_conversationId_idx" ON "corrections"("conversationId");

-- CreateIndex
CREATE INDEX "corrections_originalMessageId_idx" ON "corrections"("originalMessageId");

-- CreateIndex
CREATE INDEX "corrections_issueType_idx" ON "corrections"("issueType");

-- CreateIndex
CREATE INDEX "error_patterns_diagramType_idx" ON "error_patterns"("diagramType");

-- CreateIndex
CREATE INDEX "error_patterns_frequency_idx" ON "error_patterns"("frequency");

-- CreateIndex
CREATE UNIQUE INDEX "error_patterns_errorSignature_userId_key" ON "error_patterns"("errorSignature", "userId");

-- AddForeignKey
ALTER TABLE "conversations" ADD CONSTRAINT "conversations_userId_fkey" FOREIGN KEY ("userId") REFERENCES "users"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "conversations" ADD CONSTRAINT "conversations_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES "projects"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "messages" ADD CONSTRAINT "messages_conversationId_fkey" FOREIGN KEY ("conversationId") REFERENCES "conversations"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "feedbacks" ADD CONSTRAINT "feedbacks_messageId_fkey" FOREIGN KEY ("messageId") REFERENCES "messages"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "corrections" ADD CONSTRAINT "corrections_conversationId_fkey" FOREIGN KEY ("conversationId") REFERENCES "conversations"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "corrections" ADD CONSTRAINT "corrections_originalMessageId_fkey" FOREIGN KEY ("originalMessageId") REFERENCES "messages"("id") ON DELETE CASCADE ON UPDATE CASCADE;
