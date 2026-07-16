'use client';

import { useEffect, useRef } from 'react';
import { MessageBubble } from './message-bubble';

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

interface MessageListProps {
  messages: Message[];
  isLoading: boolean;
  onFeedback: (messageId: string, rating: number) => void;
  onCorrection: (
    messageId: string,
    originalCode: string,
    correctedCode: string,
    feedback: string,
    issueType: string
  ) => void;
  onExecuteAction?: (action: any) => Promise<void>;
}

export function MessageList({
  messages,
  isLoading,
  onFeedback,
  onCorrection,
  onExecuteAction,
}: MessageListProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
      {messages.length === 0 && !isLoading && (
        <div className="text-center py-12">
          <div className="text-gray-400 text-lg mb-2">👋</div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            Start a conversation
          </h3>
          <p className="text-sm text-gray-500 max-w-md mx-auto">
            Ask me to generate diagrams, review your architecture, or help with
            ArcLang code. I&apos;ll learn from your feedback to improve over time.
          </p>
          <div className="mt-6 space-y-2 text-left max-w-md mx-auto">
            <p className="text-xs text-gray-600 font-medium">Try asking:</p>
            <div className="space-y-1">
              <div className="text-xs text-gray-500 bg-gray-100 rounded px-3 py-2">
                &quot;Generate a system context diagram for an emergency braking system&quot;
              </div>
              <div className="text-xs text-gray-500 bg-gray-100 rounded px-3 py-2">
                &quot;Show me the allocation of functions to components&quot;
              </div>
              <div className="text-xs text-gray-500 bg-gray-100 rounded px-3 py-2">
                &quot;Review this requirement and suggest improvements&quot;
              </div>
            </div>
          </div>
        </div>
      )}

      {messages.map((message) => (
        <MessageBubble
          key={message.id}
          message={message}
          onFeedback={onFeedback}
          onCorrection={onCorrection}
          onExecuteAction={onExecuteAction}
        />
      ))}

      {isLoading && (
        <div className="flex items-start space-x-3">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-white text-sm font-medium">
              AI
            </div>
          </div>
          <div className="flex-1">
            <div className="bg-white rounded-lg shadow-sm px-4 py-3 border">
              <div className="flex space-x-2">
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce delay-100"></div>
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce delay-200"></div>
              </div>
            </div>
          </div>
        </div>
      )}

      <div ref={messagesEndRef} />
    </div>
  );
}
