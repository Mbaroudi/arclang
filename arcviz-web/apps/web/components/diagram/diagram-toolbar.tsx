'use client'

import { Button } from '@/components/ui/button'
import { ZoomIn, ZoomOut, Maximize2, Download, Filter, Layers, Share2, Settings } from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuCheckboxItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

interface DiagramToolbarProps {
  onZoomIn: () => void
  onZoomOut: () => void
  onFitView: () => void
  onExport: (format: 'png' | 'svg' | 'pdf') => void
  onFilterChange?: (filters: string[]) => void
  onLayerChange?: (layer: string) => void
  currentLayer?: string
  activeFilters?: string[]
}

export function DiagramToolbar({
  onZoomIn,
  onZoomOut,
  onFitView,
  onExport,
  onFilterChange,
  onLayerChange,
  currentLayer = 'logical',
  activeFilters = [],
}: DiagramToolbarProps) {
  const layers = [
    { id: 'operational', label: 'Operational Analysis' },
    { id: 'system', label: 'System Analysis' },
    { id: 'logical', label: 'Logical Architecture' },
    { id: 'physical', label: 'Physical Architecture' },
    { id: 'epbs', label: 'EPBS' },
  ]

  const filterOptions = [
    { id: 'requirements', label: 'Requirements' },
    { id: 'components', label: 'Components' },
    { id: 'functions', label: 'Functions' },
    { id: 'interfaces', label: 'Interfaces' },
    { id: 'traces', label: 'Traces' },
  ]

  return (
    <div className="flex items-center justify-between border-b bg-background px-4 py-2">
      <div className="flex items-center gap-2">
        {/* Layer Selection */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Layers className="mr-2 h-4 w-4" />
              {layers.find((l) => l.id === currentLayer)?.label}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            <DropdownMenuLabel>Architecture Views</DropdownMenuLabel>
            <DropdownMenuSeparator />
            {layers.map((layer) => (
              <DropdownMenuCheckboxItem
                key={layer.id}
                checked={currentLayer === layer.id}
                onCheckedChange={() => onLayerChange?.(layer.id)}
              >
                {layer.label}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Filter Menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Filter className="mr-2 h-4 w-4" />
              Filters
              {activeFilters.length > 0 && <span className="ml-1 text-xs">({activeFilters.length})</span>}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            <DropdownMenuLabel>Show Elements</DropdownMenuLabel>
            <DropdownMenuSeparator />
            {filterOptions.map((option) => (
              <DropdownMenuCheckboxItem
                key={option.id}
                checked={activeFilters.includes(option.id)}
                onCheckedChange={(checked) => {
                  const newFilters = checked
                    ? [...activeFilters, option.id]
                    : activeFilters.filter((f) => f !== option.id)
                  onFilterChange?.(newFilters)
                }}
              >
                {option.label}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="flex items-center gap-2">
        {/* Zoom Controls */}
        <div className="flex items-center gap-1 rounded-md border">
          <Button variant="ghost" size="sm" onClick={onZoomOut}>
            <ZoomOut className="h-4 w-4" />
          </Button>
          <div className="h-6 w-px bg-border" />
          <Button variant="ghost" size="sm" onClick={onFitView}>
            <Maximize2 className="h-4 w-4" />
          </Button>
          <div className="h-6 w-px bg-border" />
          <Button variant="ghost" size="sm" onClick={onZoomIn}>
            <ZoomIn className="h-4 w-4" />
          </Button>
        </div>

        {/* Export Menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Download className="mr-2 h-4 w-4" />
              Export
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => onExport('png')}>Export as PNG</DropdownMenuItem>
            <DropdownMenuItem onClick={() => onExport('svg')}>Export as SVG</DropdownMenuItem>
            <DropdownMenuItem onClick={() => onExport('pdf')}>Export as PDF</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Share Button */}
        <Button variant="ghost" size="sm">
          <Share2 className="h-4 w-4" />
        </Button>

        {/* Settings Button */}
        <Button variant="ghost" size="sm">
          <Settings className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
