'use client'

import { DiagramViewer } from '@/components/diagram/diagram-viewer'

export default function TestDiagramPage() {
  const testGraph = {
    nodes: [
      { id: 'REQ-001', label: 'Test Requirement', type: 'requirement' as const, description: 'A test requirement', safetyLevel: 'ASIL_B' },
      { id: 'COMP-001', label: 'Test Component', type: 'component' as const, description: 'A test component', safetyLevel: 'ASIL_B' },
    ],
    edges: [
      { id: 'e1', source: 'COMP-001', target: 'REQ-001', type: 'satisfies' as const },
    ],
  }

  return (
    <div className="flex h-screen flex-col">
      <header className="border-b p-4">
        <h1 className="text-xl font-bold">Diagram Viewer Test</h1>
        <p className="text-sm text-gray-600">Testing DiagramViewer component with 2 nodes</p>
      </header>
      <div className="flex-1 overflow-auto">
        <DiagramViewer
          graph={testGraph}
          width={1600}
          height={1200}
          onNodeClick={(id) => console.log('Clicked node:', id)}
          onEdgeClick={(id) => console.log('Clicked edge:', id)}
        />
      </div>
    </div>
  )
}
