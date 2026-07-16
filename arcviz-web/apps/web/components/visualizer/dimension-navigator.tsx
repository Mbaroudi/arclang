'use client';

import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  Users, 
  Target, 
  Box, 
  Cpu, 
  Package, 
  FileText, 
  Network,
  ChevronRight 
} from 'lucide-react';

export type ArcadiaDimension = 
  | 'operational' 
  | 'system' 
  | 'logical' 
  | 'physical' 
  | 'epbs' 
  | 'requirements' 
  | 'crossCutting';

interface DimensionStats {
  operational: number;
  system: number;
  logical: number;
  physical: number;
  epbs: number;
  requirements: number;
  crossCutting: number;
}

interface DimensionNavigatorProps {
  currentDimension: ArcadiaDimension;
  stats: DimensionStats;
  onDimensionChange: (dimension: ArcadiaDimension) => void;
}

const DIMENSIONS = [
  {
    id: 'operational' as ArcadiaDimension,
    name: 'Operational Analysis',
    icon: Users,
    color: 'from-blue-500 to-blue-600',
    bg: 'bg-blue-50',
    border: 'border-blue-200',
    description: 'Operational needs & activities',
    phase: '1',
  },
  {
    id: 'system' as ArcadiaDimension,
    name: 'System Analysis',
    icon: Target,
    color: 'from-green-500 to-green-600',
    bg: 'bg-green-50',
    border: 'border-green-200',
    description: 'System functions & context',
    phase: '2',
  },
  {
    id: 'logical' as ArcadiaDimension,
    name: 'Logical Architecture',
    icon: Box,
    color: 'from-purple-500 to-purple-600',
    bg: 'bg-purple-50',
    border: 'border-purple-200',
    description: 'Logical components & functions',
    phase: '3',
  },
  {
    id: 'physical' as ArcadiaDimension,
    name: 'Physical Architecture',
    icon: Cpu,
    color: 'from-orange-500 to-orange-600',
    bg: 'bg-orange-50',
    border: 'border-orange-200',
    description: 'Hardware & deployment',
    phase: '4',
  },
  {
    id: 'epbs' as ArcadiaDimension,
    name: 'EPBS',
    icon: Package,
    color: 'from-pink-500 to-pink-600',
    bg: 'bg-pink-50',
    border: 'border-pink-200',
    description: 'Product breakdown',
    phase: '5',
  },
  {
    id: 'requirements' as ArcadiaDimension,
    name: 'Requirements',
    icon: FileText,
    color: 'from-indigo-500 to-indigo-600',
    bg: 'bg-indigo-50',
    border: 'border-indigo-200',
    description: 'Traceability & verification',
    phase: '6',
  },
  {
    id: 'crossCutting' as ArcadiaDimension,
    name: 'Cross-Cutting',
    icon: Network,
    color: 'from-red-500 to-red-600',
    bg: 'bg-red-50',
    border: 'border-red-200',
    description: 'Safety, modes & states',
    phase: '7',
  },
];

export function DimensionNavigator({ 
  currentDimension, 
  stats, 
  onDimensionChange 
}: DimensionNavigatorProps) {
  const [expandedView, setExpandedView] = useState(false);

  const currentDim = DIMENSIONS.find(d => d.id === currentDimension);

  if (!expandedView) {
    // Compact view - Horizontal tabs
    return (
      <Card className="border-b rounded-none shadow-sm bg-white">
        <div className="flex items-center gap-2 px-4 py-2 overflow-x-auto">
          {DIMENSIONS.map((dim) => {
            const Icon = dim.icon;
            const isActive = dim.id === currentDimension;
            const count = stats[dim.id];
            
            return (
              <button
                key={dim.id}
                onClick={() => onDimensionChange(dim.id)}
                className={`
                  flex items-center gap-2 px-4 py-2 rounded-lg transition-all whitespace-nowrap
                  ${isActive 
                    ? `bg-gradient-to-r ${dim.color} text-white shadow-md scale-105` 
                    : `${dim.bg} hover:shadow-md hover:scale-102`
                  }
                `}
              >
                <div className="flex items-center gap-2">
                  <span className={`
                    text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center
                    ${isActive ? 'bg-white/20' : 'bg-black/10'}
                  `}>
                    {dim.phase}
                  </span>
                  <Icon className="h-4 w-4" />
                  <span className="font-medium text-sm">{dim.name}</span>
                </div>
                {count > 0 && (
                  <Badge 
                    variant={isActive ? 'secondary' : 'outline'} 
                    className="ml-1"
                  >
                    {count}
                  </Badge>
                )}
              </button>
            );
          })}
          
          <button
            onClick={() => setExpandedView(true)}
            className="ml-auto px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition-colors"
          >
            Expand →
          </button>
        </div>
      </Card>
    );
  }

  // Expanded view - Full details
  return (
    <Card className="border rounded-lg shadow-lg p-6 bg-white m-4">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">Arcadia Dimensions</h2>
        <Button 
          variant="outline" 
          size="sm"
          onClick={() => setExpandedView(false)}
        >
          Collapse ×
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {DIMENSIONS.map((dim, index) => {
          const Icon = dim.icon;
          const isActive = dim.id === currentDimension;
          const count = stats[dim.id];
          const hasNext = index < DIMENSIONS.length - 1;
          
          return (
            <div key={dim.id} className="relative">
              <button
                onClick={() => {
                  onDimensionChange(dim.id);
                  setExpandedView(false);
                }}
                className={`
                  w-full text-left p-6 rounded-xl transition-all
                  ${isActive 
                    ? `bg-gradient-to-br ${dim.color} text-white shadow-xl scale-105` 
                    : `${dim.bg} border-2 ${dim.border} hover:shadow-lg hover:scale-102`
                  }
                `}
              >
                <div className="flex items-start justify-between mb-3">
                  <div className={`
                    p-2 rounded-lg
                    ${isActive ? 'bg-white/20' : 'bg-white'}
                  `}>
                    <Icon className={`h-6 w-6 ${isActive ? 'text-white' : ''}`} />
                  </div>
                  <div className={`
                    text-xs font-bold rounded-full w-6 h-6 flex items-center justify-center
                    ${isActive ? 'bg-white/20' : 'bg-black/10'}
                  `}>
                    {dim.phase}
                  </div>
                </div>
                
                <h3 className="font-bold text-lg mb-1">{dim.name}</h3>
                <p className={`text-sm mb-3 ${isActive ? 'text-white/80' : 'text-gray-600'}`}>
                  {dim.description}
                </p>
                
                <div className="flex items-center justify-between">
                  <Badge variant={isActive ? 'secondary' : 'outline'}>
                    {count} elements
                  </Badge>
                  {isActive && (
                    <span className="text-xs font-semibold">Active ✓</span>
                  )}
                </div>
              </button>
              
              {hasNext && (
                <div className="hidden xl:block absolute top-1/2 -right-2 transform -translate-y-1/2 z-10">
                  <ChevronRight className="h-4 w-4 text-gray-400" />
                </div>
              )}
            </div>
          );
        })}
      </div>

      {currentDim && (
        <div className={`mt-6 p-4 rounded-lg ${currentDim.bg} border-2 ${currentDim.border}`}>
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg bg-gradient-to-br ${currentDim.color}`}>
              {(() => {
                const Icon = currentDim.icon;
                return <Icon className="h-5 w-5 text-white" />;
              })()}
            </div>
            <div>
              <h4 className="font-semibold">Currently Viewing: {currentDim.name}</h4>
              <p className="text-sm text-gray-600">{currentDim.description}</p>
            </div>
            <Badge className="ml-auto" variant="outline">
              Phase {currentDim.phase}/7
            </Badge>
          </div>
        </div>
      )}
    </Card>
  );
}
