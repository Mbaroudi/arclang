'use client'

import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Card } from '@/components/ui/card'
import { useToast } from '@/components/ui/use-toast'
import { Download, Loader2, Sparkles } from 'lucide-react'

const DIAGRAM_TYPES = [
  { value: 'operational', label: 'Operational Activity', icon: '🏊' },
  { value: 'functional', label: 'Functional Dataflow', icon: '🔄' },
  { value: 'component', label: 'Component Architecture', icon: '🧱' },
  { value: 'sequence', label: 'Sequence Diagram', icon: '⏱️' },
  { value: 'state-machine', label: 'State Machine', icon: '🔄' },
  { value: 'physical', label: 'Physical Architecture', icon: '🖥️' },
  { value: 'class', label: 'Class Diagram', icon: '📦' },
  { value: 'tree', label: 'Tree Diagram', icon: '🌳' },
  { value: 'capability', label: 'Capability Diagram', icon: '🎯' },
  { value: 'functional-chain', label: 'Functional Chain', icon: '⛓️' }
]

export function DiagramGenerator({ modelPath }: { modelPath: string }) {
  const [selectedType, setSelectedType] = useState('operational')
  const [loading, setLoading] = useState(false)
  const [svg, setSvg] = useState<string | null>(null)
  const [generatingAll, setGeneratingAll] = useState(false)
  const [cachedDiagrams, setCachedDiagrams] = useState<Record<string, string>>({})
  const [isCached, setIsCached] = useState(false)
  const { toast } = useToast()

  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001'

  const generateDiagram = async () => {
    const cacheKey = `${modelPath}_${selectedType}`
    if (cachedDiagrams[cacheKey]) {
      setSvg(cachedDiagrams[cacheKey])
      setIsCached(true)
      toast({
        title: 'Loaded from cache',
        description: `${selectedType} diagram loaded instantly`
      })
      return
    }
    
    setLoading(true)
    setIsCached(false)
    try {
      const response = await fetch(`${apiUrl}/api/diagrams/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ modelPath, diagramType: selectedType })
      })

      const data = await response.json()
      if (data.success) {
        setSvg(data.svg)
        setCachedDiagrams(prev => ({ ...prev, [cacheKey]: data.svg }))
        toast({
          title: 'Diagram generated',
          description: `${selectedType} diagram created successfully`
        })
      } else {
        toast({
          title: 'Generation failed',
          description: data.error || 'Unknown error',
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
      setLoading(false)
    }
  }
  
  const clearCache = () => {
    setCachedDiagrams({})
    setIsCached(false)
    setSvg(null)
    toast({
      title: 'Cache cleared',
      description: 'All cached diagrams removed'
    })
  }

  const generateAllDiagrams = async () => {
    setGeneratingAll(true)
    try {
      const response = await fetch(`${apiUrl}/api/diagrams/generate-all`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ modelPath })
      })

      const data = await response.json()
      
      if (data.success && data.diagrams && data.diagrams.length > 0) {
        const newCache: Record<string, string> = {}
        data.diagrams.forEach((diagram: any) => {
          const cacheKey = `${modelPath}_${diagram.type}`
          newCache[cacheKey] = diagram.svg
        })
        
        setCachedDiagrams(prev => ({ ...prev, ...newCache }))
        
        if (data.diagrams.length > 0) {
          const firstDiagram = data.diagrams[0]
          setSelectedType(firstDiagram.type)
          setSvg(firstDiagram.svg)
        }
        
        toast({
          title: 'All diagrams generated',
          description: `Generated ${data.successful}/10 diagrams. Switch between them using the dropdown.`
        })
      } else {
        toast({
          title: 'Generation incomplete',
          description: `Only ${data.successful || 0}/10 diagrams generated`,
          variant: 'destructive'
        })
      }
    } catch (error) {
      console.error('Failed to generate diagrams:', error)
      toast({
        title: 'Error',
        description: 'Failed to generate all diagrams',
        variant: 'destructive'
      })
    } finally {
      setGeneratingAll(false)
    }
  }

  const exportAsPNG = async () => {
    if (!svg) return
    
    try {
      const svgElement = document.createElement('div')
      svgElement.innerHTML = svg
      const svgNode = svgElement.firstElementChild as SVGElement
      
      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      
      const img = new Image()
      const svgBlob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' })
      const url = URL.createObjectURL(svgBlob)
      
      img.onload = () => {
        canvas.width = img.width * 2
        canvas.height = img.height * 2
        ctx.fillStyle = 'white'
        ctx.fillRect(0, 0, canvas.width, canvas.height)
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
        
        canvas.toBlob((blob) => {
          if (blob) {
            const link = document.createElement('a')
            link.download = `${selectedType}-diagram.png`
            link.href = URL.createObjectURL(blob)
            link.click()
          }
        })
        URL.revokeObjectURL(url)
      }
      img.src = url
      
      toast({
        title: 'Exported PNG',
        description: `${selectedType} diagram saved as PNG`
      })
    } catch (error) {
      console.error('PNG export failed:', error)
      toast({
        title: 'Export failed',
        description: 'Failed to export PNG',
        variant: 'destructive'
      })
    }
  }

  const downloadSVG = () => {
    if (!svg) return
    
    const blob = new Blob([svg], { type: 'image/svg+xml' })
    const link = document.createElement('a')
    link.download = `${selectedType}-diagram.svg`
    link.href = URL.createObjectURL(blob)
    link.click()
    
    toast({
      title: 'Downloaded SVG',
      description: `${selectedType} diagram saved`
    })
  }

  return (
    <div className="space-y-4">
      <Card className="p-4">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold">Capella Diagrams</h3>
            {Object.keys(cachedDiagrams).length > 0 && (
              <Button onClick={clearCache} size="sm" variant="ghost" className="text-xs">
                Clear ({Object.keys(cachedDiagrams).length})
              </Button>
            )}
          </div>

          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground mb-2 block">
                Select Diagram Type
              </label>
              <Select value={selectedType} onValueChange={(value) => {
                setSelectedType(value)
                const cacheKey = `${modelPath}_${value}`
                if (cachedDiagrams[cacheKey]) {
                  setSvg(cachedDiagrams[cacheKey])
                  setIsCached(true)
                } else {
                  setSvg(null)
                  setIsCached(false)
                }
              }}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select diagram type" />
                </SelectTrigger>
                <SelectContent>
                  {DIAGRAM_TYPES.map(t => (
                    <SelectItem key={t.value} value={t.value}>
                      <div className="flex items-center gap-2">
                        <span>{t.icon}</span>
                        <span>{t.label}</span>
                        {cachedDiagrams[`${modelPath}_${t.value}`] && (
                          <span className="text-xs text-green-600">✓</span>
                        )}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <Button 
                onClick={generateDiagram} 
                disabled={loading || !modelPath}
                size="sm"
                className="w-full"
              >
                {loading ? (
                  <>
                    <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    Generate Selected
                  </>
                )}
              </Button>

              <Button
                onClick={generateAllDiagrams}
                disabled={generatingAll || !modelPath}
                variant="secondary"
                size="sm"
                className="w-full"
              >
                {generatingAll ? (
                  <>
                    <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    <Sparkles className="mr-2 h-3 w-3" />
                    Generate All
                  </>
                )}
              </Button>
            </div>

            {!modelPath && (
              <p className="text-xs text-muted-foreground bg-amber-50 border border-amber-200 rounded p-2">
                💡 Save your model first to generate diagrams
              </p>
            )}
          </div>

          {svg && (
            <div className="space-y-2 pt-2 border-t">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-medium">
                    {DIAGRAM_TYPES.find(t => t.value === selectedType)?.icon}
                    {' '}
                    {DIAGRAM_TYPES.find(t => t.value === selectedType)?.label}
                  </span>
                  {isCached && (
                    <span className="text-xs text-green-600 bg-green-50 px-2 py-0.5 rounded">
                      ⚡ Cached
                    </span>
                  )}
                </div>
                <div className="flex gap-1">
                  <Button onClick={downloadSVG} size="sm" variant="outline" className="h-7 text-xs">
                    <Download className="h-3 w-3 mr-1" />
                    SVG
                  </Button>
                  <Button onClick={exportAsPNG} size="sm" variant="outline" className="h-7 text-xs">
                    <Download className="h-3 w-3 mr-1" />
                    PNG
                  </Button>
                </div>
              </div>
              
              <div className="border rounded bg-white overflow-auto max-h-[400px]">
                <div 
                  dangerouslySetInnerHTML={{ __html: svg }} 
                  className="p-2"
                  style={{ minHeight: '200px' }}
                />
              </div>
            </div>
          )}
        </div>
      </Card>

      <div className="text-xs text-muted-foreground space-y-1 px-1">
        <p className="font-medium">💡 Quick Guide:</p>
        <ul className="space-y-0.5 ml-3">
          <li>• <strong>Generate Selected</strong>: Creates the chosen diagram type</li>
          <li>• <strong>Generate All</strong>: Creates all 10 diagrams at once (cached)</li>
          <li>• Switch between cached diagrams instantly using the dropdown</li>
        </ul>
      </div>
    </div>
  )
}
