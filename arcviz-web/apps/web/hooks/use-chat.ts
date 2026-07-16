import { useState, useCallback } from 'react';

interface Message {
  id: string;
  role: 'USER' | 'ASSISTANT' | 'SYSTEM';
  content: string;
  generatedCode?: string;
  diagramSvg?: string;
  diagramType?: string;
  createdAt: string;
  feedback?: {
    rating: number;
    helpful?: boolean;
  };
  actions?: Array<{
    type: 'generate_diagram' | 'fix_code' | 'validate_code' | 'update_code';
    payload: any;
  }>;
}

interface Conversation {
  id: string;
  title?: string;
  context?: any;
  messages: Message[];
  createdAt: string;
  updatedAt: string;
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001';

export function useChat() {
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const createConversation = useCallback(async (projectId?: string) => {
    try {
      const response = await fetch(`${API_URL}/chat/conversations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ projectId }),
        credentials: 'include',
      });

      const data = await response.json();
      setConversation(data);
      setMessages(data.messages || []);
      return data;
    } catch (error) {
      console.warn('[Chat] Chat backend not available, continuing without chat');
      return null;
    }
  }, []);

  const sendMessage = useCallback(
    async (conversationId: string, content: string) => {
      setIsLoading(true);

      const userMessage: Message = {
        id: 'temp-' + Date.now(),
        role: 'USER',
        content,
        createdAt: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, userMessage]);

      try {
        const currentCode = localStorage.getItem('arcviz_current_model') || '';
        
        const context = {
          currentCode,
          codeLength: currentCode.length,
          hasCode: currentCode.length > 0,
        };

        const response = await fetch(
          `${API_URL}/chat/conversations/${conversationId}/messages`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ content, context }),
            credentials: 'include',
          }
        );

        const data = await response.json();

        setMessages((prev) => {
          const filtered = prev.filter((m) => m.id !== userMessage.id);
          return [...filtered, data.userMessage, data.assistantMessage];
        });
      } catch (error) {
        console.error('Failed to send message:', error);
        setMessages((prev) => prev.filter((m) => m.id !== userMessage.id));
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const submitFeedback = useCallback(async (messageId: string, rating: number) => {
    try {
      await fetch(`${API_URL}/chat/messages/${messageId}/feedback`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ rating }),
        credentials: 'include',
      });

      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === messageId
            ? { ...msg, feedback: { rating } }
            : msg
        )
      );
    } catch (error) {
      console.error('Failed to submit feedback:', error);
    }
  }, []);

  const submitCorrection = useCallback(
    async (
      messageId: string,
      correction: {
        originalCode: string;
        correctedCode: string;
        userFeedback: string;
        issueType: string;
      }
    ) => {
      setIsLoading(true);

      try {
        const response = await fetch(`${API_URL}/chat/messages/${messageId}/correct`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(correction),
          credentials: 'include',
        });

        const data = await response.json();
        setMessages((prev) => [...prev, data.correctedMessage]);
      } catch (error) {
        console.error('Failed to submit correction:', error);
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  return {
    conversation,
    messages,
    isLoading,
    sendMessage,
    submitFeedback,
    submitCorrection,
    createConversation,
  };
}
