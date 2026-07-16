'use client';

import { Download } from 'lucide-react';

interface DiagramPreviewProps {
  svg: string;
  diagramType?: string;
  width?: number;
  height?: number;
}

export function DiagramPreview({
  svg,
  diagramType,
  width,
  height,
}: DiagramPreviewProps) {
  const handleDownload = () => {
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.download = `${diagramType || 'diagram'}.svg`;
    link.href = url;
    link.click();
    URL.revokeObjectURL(url);
  };

  const handleInsertToVisualizer = () => {
    localStorage.setItem('arcviz_preview_diagram', svg);
    localStorage.setItem('arcviz_preview_diagram_type', diagramType || 'unknown');
    
    const event = new CustomEvent('diagram-inserted', {
      detail: { svg, diagramType },
    });
    window.dispatchEvent(event);
  };

  return (
    <div className="border rounded-lg overflow-hidden bg-gray-50">
      <div className="flex items-center justify-between px-3 py-2 bg-gray-100 border-b">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-500"></div>
          <span className="text-xs font-medium text-gray-700">
            {diagramType || 'Diagram'} Preview
          </span>
          {width && height && (
            <span className="text-xs text-gray-500">
              {width}×{height}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleInsertToVisualizer}
            className="px-2 py-1 text-xs font-medium text-blue-600 hover:bg-blue-50 rounded transition-colors"
          >
            Insert to Visualizer
          </button>
          <button
            onClick={handleDownload}
            className="p-1 text-gray-600 hover:bg-gray-200 rounded transition-colors"
            title="Download SVG"
          >
            <Download className="h-3 w-3" />
          </button>
        </div>
      </div>
      <div
        className="p-4 overflow-auto max-h-96"
        dangerouslySetInnerHTML={{ __html: svg }}
      />
    </div>
  );
}
