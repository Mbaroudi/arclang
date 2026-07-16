'use client'

import { useState, useEffect } from 'react'
import { MonacoEditor } from '@/components/editor/monaco-editor'
import { EditorToolbar } from '@/components/editor/editor-toolbar'
import { ConsolePanel } from '@/components/editor/console-panel'
import { ExportDialog } from '@/components/editor/export-dialog'
import { ImportDialog } from '@/components/editor/import-dialog'
import { ShareDialog } from '@/components/editor/share-dialog'
import { AIAssistantPanel } from '@/components/editor/ai-assistant-panel'
import { DocumentationPanel } from '@/components/editor/documentation-panel'
import { defaultArcLangCode } from '@/lib/arclang-syntax'
import { editorBridge } from '@/lib/editor-bridge'
import { editor } from 'monaco-editor'
import { useToast } from '@/components/ui/use-toast'
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Sparkles, BookOpen } from 'lucide-react'

export default function EditorPage() {
  const [code, setCode] = useState(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('arcviz_current_model')
      return saved || defaultArcLangCode
    }
    return defaultArcLangCode
  })
  const [markers, setMarkers] = useState<editor.IMarker[]>([])
  const [modelPath, setModelPath] = useState('')
  const [compilationResult, setCompilationResult] = useState<{
    success: boolean
    message: string
    stats?: {
      requirements: number
      components: number
      functions: number
      traces: number
    }
  } | undefined>()
  const [isCompiling, setIsCompiling] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showImportDialog, setShowImportDialog] = useState(false)
  const [showShareDialog, setShowShareDialog] = useState(false)
  const [rightPanelTab, setRightPanelTab] = useState<'ai' | 'docs'>('ai')
  const { toast } = useToast()
  
  // Auto-save to localStorage on code changes
  useEffect(() => {
    if (typeof window !== 'undefined' && code) {
      const timeoutId = setTimeout(() => {
        localStorage.setItem('arcviz_current_model', code)
        editorBridge.notifyCodeChanged(code)
      }, 1000)
      return () => clearTimeout(timeoutId)
    }
  }, [code])

  // Listen for code updates from chat
  useEffect(() => {
    const handleCodeUpdate = ((e: CustomEvent) => {
      const { code: newCode, source } = e.detail
      setCode(newCode)
      toast({
        title: `Code updated from ${source}`,
        description: 'Your code has been modified by the chat assistant.',
      })
    }) as EventListener

    const handleCompileRequest = ((e: CustomEvent) => {
      const { autoFix } = e.detail
      handleCompile()
      if (autoFix) {
        toast({
          title: 'Auto-fix enabled',
          description: 'Will attempt to fix errors after compilation.',
        })
      }
    }) as EventListener

    window.addEventListener('editor:reload-code', handleCodeUpdate)
    window.addEventListener('chat:compile', handleCompileRequest)

    return () => {
      window.removeEventListener('editor:reload-code', handleCodeUpdate)
      window.removeEventListener('chat:compile', handleCompileRequest)
    }
  }, [toast])

  const handleSave = async () => {
    setIsSaving(true)
    try {
      // Save to temporary file for diagram generation
      const timestamp = Date.now()
      const path = `/tmp/arcviz-${timestamp}.arc`
      
      // Store in localStorage for now
      localStorage.setItem('arcviz_current_model', code)
      setModelPath(path)
      
      await new Promise((resolve) => setTimeout(resolve, 500))
      toast({
        title: 'Saved successfully',
        description: 'Your changes have been saved.',
      })
    } catch (error) {
      toast({
        title: 'Save failed',
        description: 'Failed to save your changes.',
        variant: 'destructive',
      })
    } finally {
      setIsSaving(false)
    }
  }

  const handleCompile = async () => {
    setIsCompiling(true)
    try {
      // Simulate compilation
      await new Promise((resolve) => setTimeout(resolve, 1000))

      // Simulate compilation result
      const success = markers.filter((m) => m.severity === 8).length === 0

      setCompilationResult({
        success,
        message: success
          ? 'Compilation completed successfully'
          : 'Compilation failed with errors. Please fix the errors and try again.',
        stats: success
          ? {
              requirements: 5,
              components: 8,
              functions: 12,
              traces: 3,
            }
          : undefined,
      })

      if (success) {
        toast({
          title: 'Compilation successful',
          description: 'Your architecture has been compiled.',
        })
      }
    } catch (error) {
      toast({
        title: 'Compilation failed',
        description: 'An error occurred during compilation.',
        variant: 'destructive',
      })
    } finally {
      setIsCompiling(false)
    }
  }

  const handleVisualize = () => {
    if (!code.trim()) {
      toast({
        title: 'No code to visualize',
        description: 'Please write some ArcLang code first',
        variant: 'destructive',
      })
      return
    }
    
    toast({
      title: 'Opening visualization',
      description: 'Generating interactive architecture diagram...',
    })
    
    // Store code in localStorage to avoid URL length limits
    localStorage.setItem('arcviz_editor_code', code)
    localStorage.setItem('arcviz_editor_timestamp', Date.now().toString())
    
    // Navigate to visualization view
    window.open('/visualizer?from=editor', '_blank')
  }

  const handleExport = () => {
    setShowExportDialog(true)
  }

  const handleImport = () => {
    setShowImportDialog(true)
  }

  const handleImportComplete = (importedCode: string, fileName: string) => {
    setCode(importedCode)
    toast({
      title: 'File imported',
      description: `Loaded ${fileName} successfully`,
    })
  }

  const handleInsertAICode = (generatedCode: string) => {
    setCode(prev => prev + '\n\n' + generatedCode)
  }

  const handleShare = () => {
    setShowShareDialog(true)
  }

  const handleAIAssist = () => {
    setRightPanelTab('ai')
  }

  return (
    <div className="flex h-screen flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b bg-background px-6 py-3">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold">ArcViz Editor</h1>
          <span className="text-sm text-muted-foreground">sample_system.arc</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Last saved: 2 minutes ago</span>
        </div>
      </header>

      {/* Toolbar */}
      <EditorToolbar
        onSave={handleSave}
        onCompile={handleCompile}
        onVisualize={handleVisualize}
        onExport={handleExport}
        onImport={handleImport}
        onAIAssist={handleAIAssist}
        onShare={handleShare}
        isCompiling={isCompiling}
        isSaving={isSaving}
      />

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        <PanelGroup direction="horizontal">
          {/* Left: Editor + Console */}
          <Panel defaultSize={70} minSize={40}>
            <PanelGroup direction="vertical">
              {/* Editor Panel */}
              <Panel defaultSize={70} minSize={30}>
                <MonacoEditor value={code} onChange={(value) => setCode(value || '')} onValidate={setMarkers} />
              </Panel>

              {/* Resize Handle */}
              <PanelResizeHandle className="h-1 bg-border transition-colors hover:bg-primary" />

              {/* Console Panel */}
              <Panel defaultSize={30} minSize={20}>
                <ConsolePanel markers={markers} compilationResult={compilationResult} />
              </Panel>
            </PanelGroup>
          </Panel>

          {/* Resize Handle */}
          <PanelResizeHandle className="w-1 bg-border transition-colors hover:bg-primary" />

          {/* Right: AI Assistant & Documentation */}
          <Panel defaultSize={30} minSize={20} maxSize={40}>
            <div className="h-full border-l bg-muted/20">
              <Tabs value={rightPanelTab} onValueChange={(v) => setRightPanelTab(v as any)} className="h-full flex flex-col">
                <div className="border-b bg-background px-2 pt-2">
                  <TabsList className="grid w-full grid-cols-2">
                    <TabsTrigger value="ai" className="text-xs">
                      <Sparkles className="h-3 w-3 mr-1" />
                      AI Assistant
                    </TabsTrigger>
                    <TabsTrigger value="docs" className="text-xs">
                      <BookOpen className="h-3 w-3 mr-1" />
                      Documentation
                    </TabsTrigger>
                  </TabsList>
                </div>

                <TabsContent value="ai" className="flex-1 m-0 overflow-hidden">
                  <AIAssistantPanel currentCode={code} onInsertCode={handleInsertAICode} />
                </TabsContent>

                <TabsContent value="docs" className="flex-1 m-0 overflow-hidden">
                  <DocumentationPanel />
                </TabsContent>
              </Tabs>
            </div>
          </Panel>
        </PanelGroup>
      </div>

      {/* Dialogs */}
      <ExportDialog open={showExportDialog} onOpenChange={setShowExportDialog} code={code} />
      <ImportDialog open={showImportDialog} onOpenChange={setShowImportDialog} onImport={handleImportComplete} />
      <ShareDialog open={showShareDialog} onOpenChange={setShowShareDialog} code={code} />
    </div>
  )
}
