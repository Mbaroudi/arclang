'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useToast } from '@/components/ui/use-toast'
import { Download, Loader2, RefreshCw, ArrowLeft, MessageSquare, X, Layers } from 'lucide-react'
import Link from 'next/link'
import { ChatInterface } from '@/components/chat/chat-interface'
import { MultiDimensionVisualizer } from '@/components/visualizer/multi-dimension-visualizer'
import { CapellaDagreVisualizer } from '@/components/visualizer/capella-dagre-visualizer'

const DIAGRAM_TYPES = [
  { value: 'operational', label: 'Operational Activity', icon: '🏊', color: '#E3F2FD' },
  { value: 'functional', label: 'Functional Dataflow', icon: '🔄', color: '#F3E5F5' },
  { value: 'component', label: 'Component Architecture', icon: '🧱', color: '#E8F5E9' },
  { value: 'sequence', label: 'Sequence Diagram', icon: '⏱️', color: '#FFF3E0' },
  { value: 'state-machine', label: 'State Machine', icon: '🔄', color: '#FCE4EC' },
  { value: 'physical', label: 'Physical Architecture', icon: '🖥️', color: '#E0F2F1' },
  { value: 'class', label: 'Class Diagram', icon: '📦', color: '#F1F8E9' },
  { value: 'tree', label: 'Tree Diagram', icon: '🌳', color: '#FFF9C4' },
  { value: 'capability', label: 'Capability Diagram', icon: '🎯', color: '#E1F5FE' },
  { value: 'functional-chain', label: 'Functional Chain', icon: '⛓️', color: '#EDE7F6' }
]

interface GeneratedDiagram {
  type: string
  svg: string
  size: { width: number; height: number }
  elementCount: string
  success: boolean
}

export default function VisualizerPageIndividual() {
  const [diagrams, setDiagrams] = useState<Record<string, GeneratedDiagram>>({})
  const [generating, setGenerating] = useState<Record<string, boolean>>({})
  const [isReady, setIsReady] = useState(false)
  const [availableTypes, setAvailableTypes] = useState<string[]>([])
  const [checking, setChecking] = useState(true)
  const [chatOpen, setChatOpen] = useState(false)
  const [show7D, setShow7D] = useState(true)
  const [useHybrid, setUseHybrid] = useState(false)
  const [currentCode, setCurrentCode] = useState('')
  const { toast } = useToast()

  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001'
  const rustApiUrl = process.env.NEXT_PUBLIC_RUST_API_URL || 'http://localhost:5001'
  
  // Initialize on mount
  useEffect(() => {
    const code = localStorage.getItem('arcviz_current_model') || ''
    setCurrentCode(code)
    setChecking(false)
    setIsReady(true)
    
    // All 7 Arcadia dimensions are always available with Capella generator
    setAvailableTypes(DIAGRAM_TYPES.map(t => t.value))
  }, [])

  const generateDiagram = async (diagramType: string) => {
    if (!isReady) {
      console.log('[Visualizer] Not ready yet')
      return
    }
    
    setGenerating(prev => ({ ...prev, [diagramType]: true }))
    
    try {
      const code = localStorage.getItem('arcviz_current_model') || ''
      
      console.log(`[Visualizer] Generating ${diagramType}, code length:`, code.length)
      
      if (!code || code.trim().length < 50) {
        toast({
          title: 'No code found',
          description: 'Please write some ArcLang code in the editor first',
          variant: 'destructive'
        })
        setGenerating(prev => ({ ...prev, [diagramType]: false }))
        return
      }
      
      const response = await fetch(`${rustApiUrl}/api/diagrams/generate-professional`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ dimension: diagramType, code })
      })

      if (!response.ok) {
        const errorText = await response.text()
        console.error(`[Visualizer] API error ${response.status}:`, errorText)
        throw new Error(`API returned ${response.status}: ${errorText}`)
      }

      // Capella generator returns HTML/SVG content
      const htmlContent = await response.text()
      
      if (htmlContent && htmlContent.length > 0) {
        setDiagrams(prev => ({
          ...prev,
          [diagramType]: {
            type: diagramType,
            svg: htmlContent,
            size: { width: 3200, height: 2400 },
            elementCount: 'Capella diagram',
            success: true
          }
        }))
        
        toast({
          title: 'Diagram generated',
          description: `${DIAGRAM_TYPES.find(d => d.value === diagramType)?.label} created from your code`
        })
      } else {
        toast({
          title: 'Generation failed',
          description: 'Empty response from server',
          variant: 'destructive'
        })
      }
    } catch (error) {
      console.error('Failed to generate diagram:', error)
      toast({
        title: 'Error',
        description: 'Failed to generate diagram',
        variant: 'destructive'
      })
    } finally {
      setGenerating(prev => ({ ...prev, [diagramType]: false }))
    }
  }

  const exportDiagram = (diagram: GeneratedDiagram) => {
    const blob = new Blob([diagram.svg], { type: 'image/svg+xml' })
    const link = document.createElement('a')
    link.download = `${diagram.type}-diagram.svg`
    link.href = URL.createObjectURL(blob)
    link.click()
    URL.revokeObjectURL(link.href)
    
    toast({
      title: 'SVG Exported',
      description: `${diagram.type} diagram downloaded`
    })
  }

  const successfulDiagrams = Object.values(diagrams).filter(d => d.success)

  return (
    <div className="flex h-screen flex-col bg-gradient-to-br from-slate-50 to-slate-100 relative">
      {/* Chat Sidebar */}
      <div
        className={`fixed top-0 right-0 h-full w-full md:w-[480px] bg-white shadow-2xl transform transition-transform duration-300 ease-in-out z-50 ${
          chatOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
      >
        <div className="flex flex-col h-full">
          <div className="flex items-center justify-between px-4 py-3 border-b bg-gradient-to-r from-blue-600 to-purple-600">
            <div className="flex items-center gap-2 text-white">
              <MessageSquare className="h-5 w-5" />
              <h2 className="font-semibold">ArcLang AI Assistant</h2>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setChatOpen(false)}
              className="text-white hover:bg-white/20"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
          <div className="flex-1 overflow-hidden">
            <ChatInterface className="h-full" />
          </div>
        </div>
      </div>

      {/* Overlay */}
      {chatOpen && (
        <div
          className="fixed inset-0 bg-black/30 backdrop-blur-sm z-40 transition-opacity"
          onClick={() => setChatOpen(false)}
        />
      )}

      <header className="flex items-center justify-between border-b bg-white/80 backdrop-blur-sm px-6 py-4 shadow-sm">
        <div className="flex items-center gap-4">
          <Link href="/editor">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back to Editor
            </Button>
          </Link>
          <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
            {show7D ? 'Arcadia 7 Dimensions' : 'Capella Diagram Visualizer'}
          </h1>
          {!show7D && successfulDiagrams.length > 0 && (
            <span className="text-sm text-muted-foreground bg-slate-100 px-3 py-1 rounded-full">
              {successfulDiagrams.length} diagram{successfulDiagrams.length !== 1 ? 's' : ''} from your code
            </span>
          )}
        </div>
        
        <div className="flex items-center gap-3">
          {/* Hybrid Mode Toggle */}
          {show7D && (
            <Button
              onClick={() => setUseHybrid(!useHybrid)}
              variant={useHybrid ? "default" : "outline"}
              size="sm"
              className={useHybrid ? "bg-gradient-to-r from-green-600 to-teal-600 hover:from-green-700 hover:to-teal-700 text-white" : ""}
            >
              {useHybrid ? '🚀 Hybrid: 7D + Dagre' : 'Use Hybrid Layout'}
            </Button>
          )}
          
          {/* Toggle 7D View */}
          <Button
            onClick={() => setShow7D(!show7D)}
            variant={show7D ? "default" : "outline"}
            className={show7D ? "bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white" : ""}
          >
            <Layers className="h-4 w-4 mr-2" />
            {show7D ? '7D View Active' : 'Enable 7D View'}
          </Button>
          
          {/* AI Assistant Button */}
          <Button
            onClick={() => setChatOpen(true)}
            className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white shadow-lg hover:shadow-xl transition-all"
          >
            <MessageSquare className="h-4 w-4 mr-2" />
            AI Assistant
          </Button>
        </div>
      </header>

      {/* Floating AI Assistant Button */}
      {!chatOpen && (
        <button
          onClick={() => setChatOpen(true)}
          className="fixed bottom-6 right-6 z-30 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white rounded-full p-4 shadow-2xl hover:shadow-3xl transition-all hover:scale-110 group"
          title="Open AI Assistant"
        >
          <MessageSquare className="h-6 w-6" />
          <span className="absolute -top-1 -right-1 flex h-3 w-3">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-3 w-3 bg-purple-500"></span>
          </span>
        </button>
      )}

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-7xl mx-auto space-y-6">
          {/* 7 Dimensions View */}
          {show7D && currentCode && (
            <div className="mb-6">
              {useHybrid ? (
                <CapellaDagreVisualizer code={currentCode} dimension="operational" width={1200} height={800} />
              ) : (
                <MultiDimensionVisualizer code={currentCode} width={1000} height={700} />
              )}
            </div>
          )}
          
          {/* Regular Diagram View */}
          {!show7D && (
            <>
          {/* Diagram Type Cards */}
          {checking && (
            <Card className="p-8">
              <div className="flex items-center justify-center gap-3">
                <Loader2 className="h-5 w-5 animate-spin" />
                <p className="text-sm text-muted-foreground">Checking which diagrams can be generated from your code...</p>
              </div>
            </Card>
          )}
          
          {!checking && availableTypes.length === 0 && (
            <Card className="p-8">
              <div className="text-center">
                <div className="text-6xl mb-4">📝</div>
                <h3 className="text-lg font-semibold mb-2">No Diagrams Available</h3>
                <p className="text-sm text-muted-foreground">
                  Your code doesn&apos;t contain elements needed for diagram generation. 
                  Add actors, components, requirements, or other elements in the editor.
                </p>
              </div>
            </Card>
          )}
          
          {!checking && availableTypes.length > 0 && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
              {DIAGRAM_TYPES.filter(type => availableTypes.includes(type.value)).map((type) => {
              const isGenerating = generating[type.value]
              const hasDiagram = diagrams[type.value]?.success
              
              return (
                <Card
                  key={type.value}
                  className="overflow-hidden hover:shadow-lg transition-shadow"
                  style={{ borderTop: `4px solid ${type.color}` }}
                >
                  <CardContent className="p-4">
                    <div className="flex items-center gap-2 mb-3">
                      <span className="text-2xl">{type.icon}</span>
                      <div className="flex-1">
                        <h3 className="font-semibold text-sm">{type.label}</h3>
                        {hasDiagram && (
                          <p className="text-xs text-green-600">
                            ✓ Generated
                          </p>
                        )}
                      </div>
                    </div>
                    
                    <Button
                      onClick={() => generateDiagram(type.value)}
                      disabled={!isReady || isGenerating}
                      size="sm"
                      className="w-full"
                      variant={hasDiagram ? "outline" : "default"}
                    >
                      {isGenerating ? (
                        <>
                          <Loader2 className="h-3 w-3 mr-2 animate-spin" />
                          Generating...
                        </>
                      ) : hasDiagram ? (
                        <>
                          <RefreshCw className="h-3 w-3 mr-2" />
                          Regenerate
                        </>
                      ) : (
                        'Generate'
                      )}
                    </Button>
                  </CardContent>
                </Card>
              )
            })}
            </div>
          )}

          {/* Generated Diagrams Display */}
          {successfulDiagrams.length > 0 && (
            <div className="space-y-4">
              <h2 className="text-xl font-semibold">Generated Diagrams</h2>
              
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {successfulDiagrams.map((diagram) => {
                  const info = DIAGRAM_TYPES.find(d => d.value === diagram.type)
                  return (
                    <Card key={diagram.type} className="overflow-hidden">
                      <div className="p-4 bg-slate-50 border-b flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="text-2xl">{info?.icon}</span>
                          <div>
                            <h3 className="font-semibold">{info?.label}</h3>
                            <p className="text-xs text-muted-foreground">
                              {diagram.elementCount}
                            </p>
                          </div>
                        </div>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => exportDiagram(diagram)}
                        >
                          <Download className="h-4 w-4 mr-2" />
                          SVG
                        </Button>
                      </div>
                      
                      <div className="p-4 bg-white overflow-auto" style={{ maxHeight: '800px' }}>
                        <div
                          dangerouslySetInnerHTML={{ __html: diagram.svg }}
                          className="inline-block"
                        />
                      </div>
                    </Card>
                  )
                })}
              </div>
            </div>
          )}

          {successfulDiagrams.length === 0 && (
            <Card className="p-12">
              <div className="text-center">
                <div className="text-6xl mb-4">📊</div>
                <h3 className="text-lg font-semibold mb-2">No Diagrams Generated Yet</h3>
                <p className="text-sm text-muted-foreground">
                  Click &quot;Generate&quot; on any diagram type above to create diagrams from your ArcLang code
                </p>
              </div>
            </Card>
          )}
            </>
          )}
        </div>
      </div>
    </div>
  )
}
