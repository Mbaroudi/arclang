'use client';

import { useState, useEffect, useRef } from 'react';
import { MessageList } from './message-list';
import { ChatInput } from './chat-input';
import { useChat } from '@/hooks/use-chat';

interface ChatInterfaceProps {
  projectId?: string;
  className?: string;
}

export function ChatInterface({ projectId, className }: ChatInterfaceProps) {
  const {
    conversation,
    messages,
    isLoading,
    sendMessage,
    submitFeedback,
    submitCorrection,
    createConversation,
  } = useChat();

  const [conversationId, setConversationId] = useState<string | null>(null);

  useEffect(() => {
    if (!conversationId) {
      createConversation(projectId).then((conv) => {
        if (conv) {
          setConversationId(conv.id);
        }
      }).catch(err => {
        console.warn('[Chat] Failed to initialize chat, continuing without chat');
      });
    }
  }, [conversationId, projectId, createConversation]);

  const handleSendMessage = async (content: string) => {
    if (!conversationId) return;
    await sendMessage(conversationId, content);
  };

  const handleFeedback = async (messageId: string, rating: number) => {
    await submitFeedback(messageId, rating);
  };

  const handleCorrection = async (
    messageId: string,
    originalCode: string,
    correctedCode: string,
    feedback: string,
    issueType: string
  ) => {
    await submitCorrection(messageId, {
      originalCode,
      correctedCode,
      userFeedback: feedback,
      issueType,
    });
  };

  const handleExecuteAction = async (action: any) => {
    if (action.type === 'generate_diagram') {
      const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001';
      const code = localStorage.getItem('arcviz_current_model') || '';
      
      const response = await fetch(`${apiUrl}/api/diagrams/generate-professional`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          dimension: action.payload.diagramType, 
          code 
        })
      });
      
      if (response.ok) {
        window.dispatchEvent(new CustomEvent('chat:generate-diagram', { 
          detail: { diagramType: action.payload.diagramType, autoGenerate: true } 
        }));
      }
    } else if (action.type === 'replace_code' || action.type === 'fix_code') {
      window.dispatchEvent(new CustomEvent('chat:update-code', { 
        detail: { 
          code: action.payload.code, 
          source: 'chat',
          action: 'replace' 
        } 
      }));
    } else if (action.type === 'insert_code') {
      window.dispatchEvent(new CustomEvent('chat:update-code', { 
        detail: { 
          code: action.payload.code, 
          source: 'chat',
          action: 'insert',
          position: { section: action.payload.section }
        } 
      }));
    } else if (action.type === 'compile_code') {
      window.dispatchEvent(new CustomEvent('chat:compile', { 
        detail: { autoFix: action.payload.autoFix } 
      }));
    } else if (action.type === 'validate_code') {
      const code = localStorage.getItem('arcviz_current_model') || '';
      window.dispatchEvent(new CustomEvent('validate-code', { detail: { code } }));
    }
  };

  return (
    <div className={`flex flex-col h-full bg-gray-50 ${className}`}>
      <div className="border-b bg-white px-6 py-4">
        <h2 className="text-lg font-semibold text-gray-900">
          ArcLang Assistant
        </h2>
        <p className="text-sm text-gray-500">
          Ask questions, generate diagrams, and refine your models
        </p>
      </div>

      <MessageList
        messages={messages}
        isLoading={isLoading}
        onFeedback={handleFeedback}
        onCorrection={handleCorrection}
        onExecuteAction={handleExecuteAction}
      />

      <ChatInput
        onSend={handleSendMessage}
        disabled={isLoading || !conversationId}
        placeholder="Ask about MBSE diagrams, requirements, or corrections..."
      />
    </div>
  );
}
