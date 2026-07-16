'use client';

import { useState, useCallback, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Loader2, AlertCircle } from 'lucide-react';
import { DimensionNavigator } from './dimension-navigator';
import { parseArcadia7D, type Arcadia7DModel } from '@/lib/api-client';
import { generateProfessionalDiagram, checkRustBackendHealth } from '@/lib/rust-diagram-api';

export type ArcadiaDimension = 
  | 'operational'
  | 'system'
  | 'logical'
  | 'physical'
  | 'epbs'
  | 'requirements'
  | 'crossCutting';

interface MultiDimensionVisualizerProps {
  code: string;
  width?: number;
  height?: number;
}

export function MultiDimensionVisualizer({ 
  code, 
  width = 900, 
  height = 650 
}: MultiDimensionVisualizerProps) {
  const [currentDimension, setCurrentDimension] = useState<ArcadiaDimension>('logical');
  const [selectedNode, setSelectedNode] = useState<any>(null);
  const [model, setModel] = useState<Arcadia7DModel | null>(null);
  const [diagramHtml, setDiagramHtml] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingDiagram, setLoadingDiagram] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rustBackendAvailable, setRustBackendAvailable] = useState(false);
  
  useEffect(() => {
    let isMounted = true;
    
    async function init() {
      console.log('[7D] Code length:', code?.length || 0, 'First 100 chars:', code?.substring(0, 100));
      
      const healthy = await checkRustBackendHealth();
      if (isMounted) {
        setRustBackendAvailable(healthy);
        console.log('[Rust] Backend health:', healthy ? '✅ Available' : '❌ Unavailable');
      }
      
      if (!healthy) {
        setError('Rust backend not available on port 5001. Please run: arclang serve --port 5001');
        setLoading(false);
        return;
      }
      
      try {
        setLoading(true);
        setError(null);
        
        console.log('[7D] Loading model from Rust backend...');
        const response = await parseArcadia7D(code);
        
        if (!isMounted) return;
        
        if (response.success && response.model) {
          setModel(response.model);
          console.log('[7D] ✅ Model loaded:', response.stats);
          
          await loadDiagram(currentDimension);
        } else {
          console.error('[7D] ❌ Failed:', response.error);
          setError(`Parser error: ${response.error}`);
        }
      } catch (err) {
        console.error('[7D] Error:', err);
        if (isMounted) {
          setError('Failed to connect to Rust backend');
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    }
    
    async function loadDiagram(dimension: ArcadiaDimension) {
      if (!code || !rustBackendAvailable) return;
      
      setLoadingDiagram(true);
      const result = await generateProfessionalDiagram({ code, dimension });
      
      if (result.success && result.html) {
        setDiagramHtml(result.html);
      } else {
        console.error('[Diagram] Failed:', result.error);
      }
      setLoadingDiagram(false);
    }
    
    if (code) {
      init();
    }
    
    return () => {
      isMounted = false;
    };
  }, [code]);
  
  useEffect(() => {
    if (code && rustBackendAvailable) {
      setLoadingDiagram(true);
      generateProfessionalDiagram({ code, dimension: currentDimension }).then(result => {
        if (result.success && result.html) {
          setDiagramHtml(result.html);
        }
        setLoadingDiagram(false);
      });
    }
  }, [currentDimension, code, rustBackendAvailable]);

  const handleDimensionChange = useCallback((dimension: ArcadiaDimension) => {
    setCurrentDimension(dimension);
    setSelectedNode(null);
  }, []);

  const handleNodeClick = useCallback((node: any) => {
    setSelectedNode(node);
  }, []);

  if (loading) {
    return (
      <Card className="p-6 bg-slate-800 border-slate-700">
        <div className="flex items-center justify-center gap-3 text-slate-400">
          <Loader2 className="h-5 w-5 animate-spin" />
          <p className="text-sm">Loading professional diagrams from Rust backend...</p>
        </div>
      </Card>
    );
  }
  
  if (error || !model) {
    return (
      <Card className="p-6 bg-slate-800 border-slate-700">
        <div className="text-center text-red-400">
          <AlertCircle className="h-12 w-12 mx-auto mb-3" />
          <p className="text-lg font-semibold mb-2">Rust Backend Error</p>
          <p className="text-sm mb-4">{error || 'Unable to connect to professional diagram service'}</p>
          <p className="text-xs text-slate-500 font-mono">
            Run: cd /Users/malek/Arclang && ./target/release/arclang serve --port 5001
          </p>
        </div>
      </Card>
    );
  }

  const stats = model ? {
    operational: (model.operational?.actors?.length || 0) + (model.operational?.activities?.length || 0),
    system: (model.system?.functions?.length || 0) + (model.system?.actors?.length || 0),
    logical: (model.logical?.components?.length || 0) + (model.logical?.interfaces?.length || 0),
    physical: (model.physical?.nodes?.length || 0) + (model.physical?.links?.length || 0),
    epbs: (model.epbs?.subsystems?.length || 0) + (model.epbs?.assemblies?.length || 0),
    requirements: model.requirements?.requirements?.length || 0,
    crossCutting: (model.crossCutting?.securityPolicies?.length || 0) + (model.crossCutting?.safetyConstraints?.length || 0),
  } : {
    operational: 0,
    system: 0,
    logical: 0,
    physical: 0,
    epbs: 0,
    requirements: 0,
    crossCutting: 0,
  };

  return (
    <div className="space-y-4">
      <DimensionNavigator
        currentDimension={currentDimension}
        onDimensionChange={handleDimensionChange}
        stats={stats}
      />

      <Card className="p-0 bg-white border-slate-700 overflow-hidden">
        {loadingDiagram ? (
          <div className="flex items-center justify-center gap-3 text-slate-600 py-12">
            <Loader2 className="h-6 w-6 animate-spin" />
            <p className="text-sm">Generating professional {currentDimension} diagram...</p>
          </div>
        ) : diagramHtml ? (
          <iframe
            className="w-full min-h-[600px] border-0"
            srcDoc={diagramHtml}
            sandbox="allow-scripts allow-same-origin"
            style={{ height: '800px', border: 'none' }}
          />
        ) : (
          <div className="p-12 text-center text-slate-500">
            <p>No diagram available for this dimension</p>
          </div>
        )}
      </Card>

      {selectedNode && (
        <Card className="p-4 bg-slate-800 border-slate-700">
          <h3 className="text-lg font-semibold text-white mb-3">Node Details</h3>
          <div className="space-y-2 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-slate-400 font-medium">ID:</span>
              <span className="text-white">{selectedNode.id}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-slate-400 font-medium">Name:</span>
              <span className="text-white">{selectedNode.name}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-slate-400 font-medium">Type:</span>
              <span className="text-white capitalize">{selectedNode.type}</span>
            </div>
            {selectedNode.description && (
              <div className="flex flex-col gap-1">
                <span className="text-slate-400 font-medium">Description:</span>
                <span className="text-slate-300">{selectedNode.description}</span>
              </div>
            )}
            {selectedNode.properties && Object.keys(selectedNode.properties).length > 0 && (
              <div className="flex flex-col gap-1">
                <span className="text-slate-400 font-medium">Properties:</span>
                <div className="pl-3 space-y-1">
                  {Object.entries(selectedNode.properties).map(([key, value]) => (
                    <div key={key} className="flex items-center gap-2">
                      <span className="text-slate-400 text-xs">{key}:</span>
                      <span className="text-slate-300 text-xs">{String(value)}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}
