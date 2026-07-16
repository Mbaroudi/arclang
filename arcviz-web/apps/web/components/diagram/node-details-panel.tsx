'use client'

import { X, ExternalLink, GitBranch, Shield, FileText } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ArchitectureNode } from '@/lib/elk/elk-layout'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'

interface NodeDetailsPanelProps {
  node: ArchitectureNode | null
  onClose: () => void
}

export function NodeDetailsPanel({ node, onClose }: NodeDetailsPanelProps) {
  if (!node) return null

  const safetyLevelColors: Record<string, string> = {
    ASIL_D: 'bg-red-100 text-red-800 border-red-300',
    ASIL_C: 'bg-orange-100 text-orange-800 border-orange-300',
    ASIL_B: 'bg-yellow-100 text-yellow-800 border-yellow-300',
    ASIL_A: 'bg-green-100 text-green-800 border-green-300',
    DAL_A: 'bg-red-100 text-red-800 border-red-300',
    DAL_B: 'bg-orange-100 text-orange-800 border-orange-300',
    DAL_C: 'bg-yellow-100 text-yellow-800 border-yellow-300',
    DAL_D: 'bg-green-100 text-green-800 border-green-300',
    QM: 'bg-blue-100 text-blue-800 border-blue-300',
  }

  return (
    <div className="flex h-full flex-col border-l bg-background">
      {/* Header */}
      <div className="flex items-center justify-between border-b p-4">
        <h3 className="font-semibold">Element Details</h3>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {/* ID Badge */}
        <div className="mb-4">
          <Badge variant="outline" className="text-sm">
            {node.id}
          </Badge>
        </div>

        {/* Title */}
        <h2 className="mb-2 text-xl font-bold">{node.label}</h2>

        {/* Type */}
        <p className="mb-4 text-sm text-muted-foreground capitalize">{node.type}</p>

        <Separator className="my-4" />

        {/* Safety Level */}
        {node.safetyLevel && (
          <div className="mb-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <Shield className="h-4 w-4" />
              Safety Level
            </div>
            <Badge className={safetyLevelColors[node.safetyLevel] || 'bg-gray-100 text-gray-800'}>
              {node.safetyLevel}
            </Badge>
          </div>
        )}

        {/* Description */}
        {node.description && (
          <div className="mb-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <FileText className="h-4 w-4" />
              Description
            </div>
            <p className="text-sm text-muted-foreground">{node.description}</p>
          </div>
        )}

        {/* Layer Information */}
        {node.layer && (
          <div className="mb-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <GitBranch className="h-4 w-4" />
              Architectural Layer
            </div>
            <Badge variant="secondary" className="capitalize">
              {node.layer}
            </Badge>
          </div>
        )}

        {/* Functions/Children */}
        {node.children && node.children.length > 0 && (
          <div className="mb-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <FileText className="h-4 w-4" />
              Functions ({node.children.length})
            </div>
            <div className="space-y-2 max-h-60 overflow-y-auto">
              {node.children.map((child, index) => (
                <div key={index} className="rounded-md bg-muted p-3 text-sm hover:bg-muted/80 transition-colors">
                  <div className="font-semibold text-primary">{child.label}</div>
                  {child.description && (
                    <div className="text-xs text-muted-foreground mt-1">{child.description}</div>
                  )}
                  {child.id && (
                    <div className="text-xs text-muted-foreground mt-1 font-mono">ID: {child.id}</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        <Separator className="my-4" />
        
        {/* Technical Properties */}
        <div className="mb-4">
          <div className="mb-2 text-sm font-medium">Technical Properties</div>
          <div className="space-y-1 text-xs bg-muted/30 rounded-md p-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Node ID:</span>
              <span className="font-mono font-semibold">{node.id}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Type:</span>
              <span className="capitalize">{node.type}</span>
            </div>
            {node.width && node.height && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Dimensions:</span>
                <span className="font-mono">{Math.round(node.width)}×{Math.round(node.height)}px</span>
              </div>
            )}
          </div>
        </div>

        <Separator className="my-4" />

        {/* Traces (Sample) */}
        <div className="mb-4">
          <div className="mb-2 flex items-center gap-2 text-sm font-medium">
            <GitBranch className="h-4 w-4" />
            Traceability
          </div>
          <div className="space-y-2">
            <div className="rounded-md border p-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="font-medium">Satisfies REQ-001</span>
                <Button variant="ghost" size="sm">
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>
              <p className="mt-1 text-xs text-muted-foreground">System shall process sensor data</p>
            </div>
            <div className="rounded-md border p-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="font-medium">Implements SF-001</span>
                <Button variant="ghost" size="sm">
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>
              <p className="mt-1 text-xs text-muted-foreground">Data processing function</p>
            </div>
          </div>
        </div>

        <Separator className="my-4" />

        {/* Properties */}
        <div>
          <div className="mb-2 text-sm font-medium">Properties</div>
          <dl className="space-y-2 text-sm">
            <div className="flex justify-between">
              <dt className="text-muted-foreground">Created</dt>
              <dd className="font-medium">2024-01-15</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-muted-foreground">Modified</dt>
              <dd className="font-medium">2024-01-20</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-muted-foreground">Author</dt>
              <dd className="font-medium">John Doe</dd>
            </div>
          </dl>
        </div>
      </div>

      {/* Footer Actions */}
      <div className="border-t p-4">
        <Button className="w-full" variant="outline">
          <FileText className="mr-2 h-4 w-4" />
          View in Editor
        </Button>
      </div>
    </div>
  )
}
