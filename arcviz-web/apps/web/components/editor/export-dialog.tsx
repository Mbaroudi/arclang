'use client'

import { useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Download, FileCode, FileJson, Network, Loader2 } from 'lucide-react'
import { compileApi } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'

interface ExportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  code: string
}

export function ExportDialog({ open, onOpenChange, code }: ExportDialogProps) {
  const [exportFormat, setExportFormat] = useState<'capella' | 'mermaid' | 'json'>('capella')
  const [isExporting, setIsExporting] = useState(false)
  const { toast } = useToast()

  const handleExport = async () => {
    if (!code.trim()) {
      toast({
        title: 'No code to export',
        description: 'Please write some ArcLang code first',
        variant: 'destructive',
      })
      return
    }

    setIsExporting(true)

    try {
      switch (exportFormat) {
        case 'capella':
          const capellaBlob = await compileApi.exportCapella(code)
          const capellaUrl = URL.createObjectURL(capellaBlob)
          const capellaLink = document.createElement('a')
          capellaLink.href = capellaUrl
          capellaLink.download = `architecture-${Date.now()}.capella`
          capellaLink.click()
          URL.revokeObjectURL(capellaUrl)
          
          toast({
            title: 'Capella XML Exported',
            description: 'Architecture exported in Capella format',
          })
          break

        case 'mermaid':
          const mermaidCode = await compileApi.exportMermaid(code)
          const mermaidBlob = new Blob([mermaidCode], { type: 'text/plain' })
          const mermaidUrl = URL.createObjectURL(mermaidBlob)
          const mermaidLink = document.createElement('a')
          mermaidLink.href = mermaidUrl
          mermaidLink.download = `architecture-${Date.now()}.mmd`
          mermaidLink.click()
          URL.revokeObjectURL(mermaidUrl)
          
          toast({
            title: 'Mermaid Diagram Exported',
            description: 'Architecture exported as Mermaid diagram',
          })
          break

        case 'json':
          const jsonData = await compileApi.exportJSON(code)
          const jsonBlob = new Blob([JSON.stringify(jsonData, null, 2)], { type: 'application/json' })
          const jsonUrl = URL.createObjectURL(jsonBlob)
          const jsonLink = document.createElement('a')
          jsonLink.href = jsonUrl
          jsonLink.download = `architecture-${Date.now()}.json`
          jsonLink.click()
          URL.revokeObjectURL(jsonUrl)
          
          toast({
            title: 'JSON Exported',
            description: 'Architecture exported as JSON',
          })
          break
      }

      onOpenChange(false)
    } catch (error: any) {
      toast({
        title: 'Export failed',
        description: error.response?.data?.error || error.message || 'Failed to export',
        variant: 'destructive',
      })
    } finally {
      setIsExporting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Export Architecture</DialogTitle>
          <DialogDescription>
            Export your architecture model to various formats compatible with industry-standard tools
          </DialogDescription>
        </DialogHeader>

        <Tabs defaultValue="capella" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="capella" onClick={() => setExportFormat('capella')}>
              Capella XML
            </TabsTrigger>
            <TabsTrigger value="mermaid" onClick={() => setExportFormat('mermaid')}>
              Mermaid
            </TabsTrigger>
            <TabsTrigger value="json" onClick={() => setExportFormat('json')}>
              JSON
            </TabsTrigger>
          </TabsList>

          <TabsContent value="capella" className="space-y-4">
            <div className="flex items-start gap-3 rounded-lg border p-4">
              <FileCode className="h-5 w-5 text-blue-600 mt-0.5" />
              <div className="flex-1">
                <h4 className="font-semibold text-sm">Capella XML Format</h4>
                <p className="text-xs text-muted-foreground mt-1">
                  Export to Eclipse Capella format for use with Capella MBSE tool. Includes all layers, components, functions, and traceability links.
                </p>
                <div className="mt-2 text-xs">
                  <span className="font-semibold">Compatible with:</span> Eclipse Capella, Arcadia MBSE
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="mermaid" className="space-y-4">
            <div className="flex items-start gap-3 rounded-lg border p-4">
              <Network className="h-5 w-5 text-purple-600 mt-0.5" />
              <div className="flex-1">
                <h4 className="font-semibold text-sm">Mermaid Diagram</h4>
                <p className="text-xs text-muted-foreground mt-1">
                  Export as Mermaid diagram code for documentation and presentations. Can be rendered in Markdown, Notion, GitHub, GitLab.
                </p>
                <div className="mt-2 text-xs">
                  <span className="font-semibold">Compatible with:</span> GitHub, GitLab, Notion, Obsidian
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="json" className="space-y-4">
            <div className="flex items-start gap-3 rounded-lg border p-4">
              <FileJson className="h-5 w-5 text-green-600 mt-0.5" />
              <div className="flex-1">
                <h4 className="font-semibold text-sm">JSON Format</h4>
                <p className="text-xs text-muted-foreground mt-1">
                  Export as structured JSON for custom integrations, APIs, and data processing. Includes complete model graph with nodes and edges.
                </p>
                <div className="mt-2 text-xs">
                  <span className="font-semibold">Use cases:</span> API integration, Custom tools, Data analysis
                </div>
              </div>
            </div>
          </TabsContent>
        </Tabs>

        <div className="flex justify-end gap-2 mt-4">
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isExporting}>
            Cancel
          </Button>
          <Button onClick={handleExport} disabled={isExporting}>
            {isExporting ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <Download className="mr-2 h-4 w-4" />
                Export
              </>
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
