'use client'

import { useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Label } from '@/components/ui/label'
import { Upload, FileCode, Loader2 } from 'lucide-react'
import { useToast } from '@/components/ui/use-toast'

interface ImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onImport: (code: string, fileName: string) => void
}

export function ImportDialog({ open, onOpenChange, onImport }: ImportDialogProps) {
  const [isImporting, setIsImporting] = useState(false)
  const [dragOver, setDragOver] = useState(false)
  const { toast } = useToast()

  const handleFileUpload = async (file: File) => {
    if (!file) return

    setIsImporting(true)

    try {
      const text = await file.text()

      if (file.name.endsWith('.arc')) {
        onImport(text, file.name)
        toast({
          title: 'File imported',
          description: `Loaded ${file.name} successfully`,
        })
      } else if (file.name.endsWith('.capella') || file.name.endsWith('.xml')) {
        toast({
          title: 'Capella XML Import',
          description: 'Converting Capella XML to ArcLang...',
        })
        
        const convertedCode = `// Imported from ${file.name}\n// TODO: Capella XML parsing implementation\n\n${text.substring(0, 500)}...`
        onImport(convertedCode, file.name)
      } else if (file.name.endsWith('.json')) {
        const jsonData = JSON.parse(text)
        const code = `// Imported from ${file.name}\n// JSON structure detected\n\n`
        onImport(code, file.name)
        toast({
          title: 'JSON imported',
          description: `Loaded ${file.name}`,
        })
      } else {
        toast({
          title: 'Unsupported format',
          description: 'Please upload .arc, .capella, .xml, or .json files',
          variant: 'destructive',
        })
        setIsImporting(false)
        return
      }

      onOpenChange(false)
    } catch (error: any) {
      toast({
        title: 'Import failed',
        description: error.message || 'Failed to read file',
        variant: 'destructive',
      })
    } finally {
      setIsImporting(false)
    }
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)

    const files = Array.from(e.dataTransfer.files)
    if (files.length > 0) {
      handleFileUpload(files[0])
    }
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(true)
  }

  const handleDragLeave = () => {
    setDragOver(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Import Architecture</DialogTitle>
          <DialogDescription>
            Import architecture models from various formats
          </DialogDescription>
        </DialogHeader>

        <div
          className={`relative border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
            dragOver ? 'border-primary bg-primary/5' : 'border-muted-foreground/25'
          }`}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
        >
          <input
            type="file"
            id="file-upload"
            className="hidden"
            accept=".arc,.capella,.xml,.json"
            onChange={(e) => {
              const file = e.target.files?.[0]
              if (file) handleFileUpload(file)
            }}
            disabled={isImporting}
          />

          <div className="flex flex-col items-center gap-3">
            <div className="rounded-full bg-primary/10 p-3">
              {isImporting ? (
                <Loader2 className="h-6 w-6 text-primary animate-spin" />
              ) : (
                <Upload className="h-6 w-6 text-primary" />
              )}
            </div>

            <div>
              <p className="text-sm font-semibold">
                {isImporting ? 'Importing file...' : 'Drag and drop your file here'}
              </p>
              <p className="text-xs text-muted-foreground mt-1">or</p>
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => document.getElementById('file-upload')?.click()}
              disabled={isImporting}
            >
              <FileCode className="mr-2 h-4 w-4" />
              Browse Files
            </Button>
          </div>
        </div>

        <div className="space-y-2">
          <Label className="text-sm font-semibold">Supported Formats</Label>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div className="flex items-center gap-2 rounded-md border p-2">
              <div className="h-2 w-2 rounded-full bg-blue-500" />
              <span>.arc (ArcLang)</span>
            </div>
            <div className="flex items-center gap-2 rounded-md border p-2">
              <div className="h-2 w-2 rounded-full bg-green-500" />
              <span>.capella (Capella XML)</span>
            </div>
            <div className="flex items-center gap-2 rounded-md border p-2">
              <div className="h-2 w-2 rounded-full bg-orange-500" />
              <span>.xml (Generic XML)</span>
            </div>
            <div className="flex items-center gap-2 rounded-md border p-2">
              <div className="h-2 w-2 rounded-full bg-purple-500" />
              <span>.json (JSON)</span>
            </div>
          </div>
        </div>

        <div className="flex justify-end">
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isImporting}>
            Cancel
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
