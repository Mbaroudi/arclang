'use client';

import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { CheckCircle2, Code, FileText, Sparkles } from 'lucide-react';
import { useState } from 'react';

interface MessageAction {
  type: 'generate_diagram' | 'fix_code' | 'validate_code' | 'update_code' | 'insert_code' | 'replace_code' | 'compile_code';
  payload: any;
}

interface MessageActionsProps {
  actions: MessageAction[];
  onExecuteAction: (action: MessageAction) => Promise<void>;
}

export function MessageActions({ actions, onExecuteAction }: MessageActionsProps) {
  const [executing, setExecuting] = useState<string | null>(null);
  const [executed, setExecuted] = useState<Set<string>>(new Set());

  if (!actions || actions.length === 0) return null;

  const handleExecute = async (action: MessageAction, index: number) => {
    const key = `${action.type}-${index}`;
    setExecuting(key);
    try {
      await onExecuteAction(action);
      setExecuted(prev => new Set(prev).add(key));
    } finally {
      setExecuting(null);
    }
  };

  const getActionConfig = (type: string) => {
    switch (type) {
      case 'generate_diagram':
        return {
          icon: Sparkles,
          label: 'Generate Diagram',
          color: 'bg-blue-500 hover:bg-blue-600',
        };
      case 'fix_code':
      case 'replace_code':
        return {
          icon: Code,
          label: 'Replace Code',
          color: 'bg-green-500 hover:bg-green-600',
        };
      case 'insert_code':
        return {
          icon: FileText,
          label: 'Insert Code',
          color: 'bg-indigo-500 hover:bg-indigo-600',
        };
      case 'compile_code':
        return {
          icon: CheckCircle2,
          label: 'Compile',
          color: 'bg-purple-500 hover:bg-purple-600',
        };
      case 'validate_code':
        return {
          icon: CheckCircle2,
          label: 'Validate Code',
          color: 'bg-purple-500 hover:bg-purple-600',
        };
      case 'update_code':
        return {
          icon: FileText,
          label: 'Update Code',
          color: 'bg-orange-500 hover:bg-orange-600',
        };
      default:
        return {
          icon: Sparkles,
          label: 'Execute Action',
          color: 'bg-gray-500 hover:bg-gray-600',
        };
    }
  };

  return (
    <Card className="mt-3 p-3 bg-gradient-to-r from-blue-50 to-purple-50 border-blue-200">
      <div className="flex flex-col gap-2">
        <p className="text-xs font-semibold text-gray-600 mb-1">
          💡 Suggested Actions
        </p>
        {actions.map((action, index) => {
          const key = `${action.type}-${index}`;
          const config = getActionConfig(action.type);
          const Icon = config.icon;
          const isExecuting = executing === key;
          const isExecuted = executed.has(key);

          return (
            <Button
              key={key}
              onClick={() => handleExecute(action, index)}
              disabled={isExecuting || isExecuted}
              size="sm"
              className={`${config.color} text-white justify-start ${
                isExecuted ? 'opacity-50' : ''
              }`}
            >
              <Icon className="h-4 w-4 mr-2" />
              {isExecuting
                ? 'Executing...'
                : isExecuted
                ? '✓ ' + config.label
                : config.label}
              {action.payload.diagramType && (
                <span className="ml-2 text-xs opacity-80">
                  ({action.payload.diagramType})
                </span>
              )}
            </Button>
          );
        })}
      </div>
    </Card>
  );
}
