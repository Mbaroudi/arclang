'use client';

import { ChatInterface } from '@/components/chat/chat-interface';
import { useSearchParams } from 'next/navigation';

export default function ChatPage() {
  const searchParams = useSearchParams();
  const projectId = searchParams?.get('projectId') || undefined;

  return (
    <div className="h-screen flex flex-col">
      <header className="bg-white border-b px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">
              ArcLang AI Assistant
            </h1>
            <p className="text-sm text-gray-500">
              Conversational MBSE diagram generation with learning
            </p>
          </div>
          <div className="flex items-center space-x-4">
            <a
              href="/editor"
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border rounded-lg hover:bg-gray-50"
            >
              Editor
            </a>
            <a
              href="/visualizer"
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border rounded-lg hover:bg-gray-50"
            >
              Visualizer
            </a>
          </div>
        </div>
      </header>

      <main className="flex-1 overflow-hidden">
        <ChatInterface projectId={projectId} className="h-full" />
      </main>
    </div>
  );
}
