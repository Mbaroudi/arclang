'use client';

import { useState } from 'react';
import { FeedbackButtons } from './feedback-buttons';
import { CorrectionEditor } from './correction-editor';
import { CodeBlock } from './code-block';
import { DiagramPreview } from './diagram-preview';
import { MessageActions } from './message-actions';

interface Message {
  id: string;
  role: 'USER' | 'ASSISTANT' | 'SYSTEM';
  content: string;
  generatedCode?: string;
  diagramSvg?: string;
  diagramType?: string;
  feedback?: {
    rating: number;
  };
  actions?: Array<{
    type: 'generate_diagram' | 'fix_code' | 'validate_code' | 'update_code';
    payload: any;
  }>;
}

interface MessageBubbleProps {
  message: Message;
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

export function MessageBubble({
  message,
  onFeedback,
  onCorrection,
  onExecuteAction,
}: MessageBubbleProps) {
  const [showCorrectionEditor, setShowCorrectionEditor] = useState(false);

  const isUser = message.role === 'USER';
  const isAssistant = message.role === 'ASSISTANT';

  return (
    <div className={`flex items-start space-x-3 ${isUser ? 'flex-row-reverse space-x-reverse' : ''}`}>
      <div className="flex-shrink-0">
        <div
          className={`w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-medium ${
            isUser ? 'bg-green-500' : 'bg-blue-500'
          }`}
        >
          {isUser ? 'U' : 'AI'}
        </div>
      </div>

      <div className="flex-1 max-w-3xl">
        <div
          className={`rounded-lg shadow-sm px-4 py-3 border ${
            isUser
              ? 'bg-green-50 border-green-200'
              : 'bg-white border-gray-200'
          }`}
        >
          <div className="prose prose-sm max-w-none">
            {message.content.split('\n').map((line, i) => (
              <p key={i} className="mb-2 last:mb-0">
                {line}
              </p>
            ))}
          </div>

          {message.generatedCode && (
            <div className="mt-4">
              <CodeBlock code={message.generatedCode} language="arclang" />
            </div>
          )}

          {message.diagramSvg && (
            <div className="mt-4">
              <DiagramPreview
                svg={message.diagramSvg}
                diagramType={message.diagramType}
              />
            </div>
          )}
        </div>

        {isAssistant && message.actions && message.actions.length > 0 && onExecuteAction && (
          <MessageActions
            actions={message.actions}
            onExecuteAction={onExecuteAction}
          />
        )}

        {isAssistant && (
          <div className="mt-2">
            <FeedbackButtons
              messageId={message.id}
              currentRating={message.feedback?.rating}
              onFeedback={onFeedback}
              onRequestCorrection={() => setShowCorrectionEditor(true)}
            />
          </div>
        )}

        {showCorrectionEditor && (
          <div className="mt-3">
            <CorrectionEditor
              messageId={message.id}
              originalCode={message.generatedCode || ''}
              onSubmit={(correctedCode, feedback, issueType) => {
                onCorrection(
                  message.id,
                  message.generatedCode || '',
                  correctedCode,
                  feedback,
                  issueType
                );
                setShowCorrectionEditor(false);
              }}
              onCancel={() => setShowCorrectionEditor(false)}
            />
          </div>
        )}
      </div>
    </div>
  );
}
