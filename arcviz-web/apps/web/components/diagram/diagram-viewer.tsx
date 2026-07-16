'use client'

import { useEffect, useRef, useState, useMemo } from 'react'
import * as d3 from 'd3'
import { ElkNode } from 'elkjs/lib/elk.bundled.js'
import {
  ArchitectureGraph,
  DiagramType,
  layoutGraph,
  calculateNodeDimensions,
  getNodeColor,
  getEdgeColor,
} from '@/lib/elk/elk-layout'
import { ARCVIZ_CONFIG } from '@/lib/arcviz-config'
import { Loader2 } from 'lucide-react'

interface DiagramViewerProps {
  graph: ArchitectureGraph
  diagramType?: DiagramType
  width?: number
  height?: number
  onNodeClick?: (nodeId: string) => void
  onEdgeClick?: (edgeId: string) => void
}

export function DiagramViewer({ graph, diagramType, width = 1200, height = 800, onNodeClick, onEdgeClick }: DiagramViewerProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const renderingRef = useRef(false)
  const zoomBehaviorRef = useRef<any>(null)
  const [zoomLevel, setZoomLevel] = useState(0.8)

  // Create stable graph key to prevent unnecessary re-renders
  const graphKey = useMemo(
    () => `${graph.nodes.length}-${graph.edges.length}`,
    [graph.nodes.length, graph.edges.length]
  )

  useEffect(() => {
    // Prevent multiple simultaneous renders
    if (renderingRef.current) return
    
    const checkAndRender = async () => {
      renderingRef.current = true
      
      // Wait for SVG to be mounted
      if (!svgRef.current) {
        for (let i = 0; i < 10; i++) {
          if (svgRef.current) break
          await new Promise(resolve => setTimeout(resolve, 100))
        }
      }
      
      if (!svgRef.current) {
        setError('Failed to initialize SVG element')
        setIsLoading(false)
        renderingRef.current = false
        return
      }
      
      await renderDiagram()
      renderingRef.current = false
    }

    const renderDiagram = async () => {
      try {
        setIsLoading(true)
        setError(null)

        // Calculate node dimensions
        const graphWithDimensions = {
          ...graph,
          nodes: graph.nodes.map((node) => {
            const dims = calculateNodeDimensions(node)
            console.log(`Node ${node.id}: ${dims.width}x${dims.height}`)
            return {
              ...node,
              ...dims,
            }
          }),
        }

        console.log('DiagramViewer: Running ELK layout with diagram type:', diagramType)

        // Run ELK layout with timeout
        let layouted
        try {
          const layoutPromise = layoutGraph(graphWithDimensions, diagramType)
          const timeoutPromise = new Promise((_, reject) => 
            setTimeout(() => reject(new Error('Layout timeout after 10s')), 10000)
          )
          
          layouted = await Promise.race([layoutPromise, timeoutPromise]) as any
          console.log('DiagramViewer: Layout complete, layouted.children:', layouted.children?.length || 0)
        } catch (layoutError: any) {
          console.error('DiagramViewer: ELK layout failed:', layoutError)
          throw new Error(`Layout failed: ${layoutError.message}`)
        }

        // Clear previous content
        const svg = d3.select(svgRef.current)
        svg.selectAll('*').remove()
        
        // Add white background for exports
        svg.append('rect')
          .attr('width', '100%')
          .attr('height', '100%')
          .attr('fill', 'white')
          .style('pointer-events', 'none')

        // Create main group with zoom behavior
        const g = svg.append('g')

        const zoom = d3
          .zoom<SVGSVGElement, unknown>()
          .scaleExtent([0.1, 4])
          .wheelDelta((event: any) => {
            // Smooth zoom like Google Maps
            return -event.deltaY * (event.deltaMode === 1 ? 0.05 : event.deltaMode ? 1 : 0.002)
          })
          .on('zoom', (event) => {
            g.attr('transform', event.transform)
            setZoomLevel(event.transform.k)
          })

        // Enable smooth panning and zooming
        svg.call(zoom as any)
        
        // Add cursor styles for better UX
        svg.style('cursor', 'grab')
        svg.on('mousedown', function() {
          d3.select(this).style('cursor', 'grabbing')
        })
        svg.on('mouseup', function() {
          d3.select(this).style('cursor', 'grab')
        })
        svg.on('mouseleave', function() {
          d3.select(this).style('cursor', 'grab')
        })
        
        // Store zoom behavior for external control
        zoomBehaviorRef.current = { svg, zoom }

        // Initial transform to center
        const initialScale = 0.8
        const centerX = width / 2 - ((layouted.width || 0) * initialScale) / 2
        const centerY = 50
        svg.call(zoom.transform as any, d3.zoomIdentity.translate(centerX, centerY).scale(initialScale))

        // Determine rendering style based on diagram type
        const isDataflow = diagramType === 'ldfb' || diagramType === 'sdfb'
        const isBreakdown = diagramType === 'lcbd' || diagramType === 'scbd'

        // Create arrow markers for edges
        const defs = svg.append('defs')
        const edgeTypes = ['satisfies', 'implements', 'realizes', 'data']
        edgeTypes.forEach((type) => {
          const color = getEdgeColor({ id: '', source: '', target: '', type: type as any })
          const markerSize = isDataflow ? 8 : 6  // Larger markers for dataflow
          defs
            .append('marker')
            .attr('id', `arrow-${type}`)
            .attr('viewBox', '0 0 10 10')
            .attr('refX', 9)
            .attr('refY', 5)
            .attr('markerWidth', markerSize)
            .attr('markerHeight', markerSize)
            .attr('orient', 'auto')
            .append('path')
            .attr('d', 'M 0 0 L 10 5 L 0 10 z')
            .attr('fill', color)
        })

        // Draw edges
        const edges = g.append('g').attr('class', 'edges')

        layouted.edges?.forEach((elkEdge: any) => {
          const edge = graph.edges.find((e) => e.id === elkEdge.id)
          if (!edge || !elkEdge.sections || elkEdge.sections.length === 0) return

          const section = elkEdge.sections[0]
          const color = getEdgeColor(edge)

          // Different edge styles based on diagram type
          const edgeWidth = isDataflow ? 3 : 2  // Thicker edges for dataflow
          const edgeHoverWidth = isDataflow ? 5 : 3

          // Draw edge path
          let pathData: string
          
          if (isBreakdown) {
            // Tree-style L-shaped connectors for breakdown diagrams
            const startX = section.startPoint.x
            const startY = section.startPoint.y
            const endX = section.endPoint.x
            const endY = section.endPoint.y
            const midY = (startY + endY) / 2
            
            pathData = `M ${startX},${startY} L ${startX},${midY} L ${endX},${midY} L ${endX},${endY}`
          } else {
            // Standard path with bend points
            pathData = `M ${section.startPoint.x} ${section.startPoint.y}`
            if (section.bendPoints) {
              section.bendPoints.forEach((bp: any) => {
                pathData += ` L ${bp.x} ${bp.y}`
              })
            }
            pathData += ` L ${section.endPoint.x} ${section.endPoint.y}`
          }

          const path = edges
            .append('path')
            .attr('class', 'edge')
            .attr('d', pathData)
            .attr('fill', 'none')
            .attr('stroke', isDataflow ? '#2563eb' : color)  // Blue for dataflow
            .attr('stroke-width', edgeWidth)
            .attr('marker-end', `url(#arrow-${edge.type || 'data'})`)
            .style('cursor', 'pointer')
            .on('click', () => onEdgeClick?.(edge.id))
            .on('mouseenter', function () {
              d3.select(this).attr('stroke-width', edgeHoverWidth)
            })
            .on('mouseleave', function () {
              d3.select(this).attr('stroke-width', edgeWidth)
            })

          // Draw edge label
          if (edge.label && section.bendPoints && section.bendPoints.length > 0) {
            const midPoint = section.bendPoints[Math.floor(section.bendPoints.length / 2)]
            const labelGroup = edges.append('g')
            
            if (isDataflow) {
              // Data labels with background for dataflow
              labelGroup
                .append('rect')
                .attr('x', midPoint.x - 25)
                .attr('y', midPoint.y - 12)
                .attr('width', 50)
                .attr('height', 16)
                .attr('rx', 3)
                .attr('fill', '#dbeafe')
                .attr('stroke', '#2563eb')
                .attr('stroke-width', 1)
            }
            
            labelGroup
              .append('text')
              .attr('x', midPoint.x)
              .attr('y', midPoint.y)
              .attr('text-anchor', 'middle')
              .attr('dominant-baseline', 'middle')
              .attr('font-size', isDataflow ? '11px' : '10px')
              .attr('fill', isDataflow ? '#1e40af' : '#6b7280')
              .attr('font-weight', isDataflow ? '700' : '500')
              .text(edge.label)
          }
        })

        // Draw nodes
        const nodes = g.append('g').attr('class', 'nodes')

        layouted.children?.forEach((elkNode: any) => {
          const node = graph.nodes.find((n) => n.id === elkNode.id)
          if (!node) return

          const colors = getNodeColor(node)
          const nodeGroup = nodes.append('g').attr('class', 'node').attr('transform', `translate(${elkNode.x},${elkNode.y})`)

          // Node shape based on type, layer, and diagram type
          const isActor = node.type === 'actor'
          const isRequirement = node.type === 'requirement'
          
          // Add expand/collapse icon for breakdown diagrams
          if (isBreakdown && node.children && node.children.length > 0) {
            nodeGroup
              .append('circle')
              .attr('cx', elkNode.width! / 2)
              .attr('cy', elkNode.height! + 10)
              .attr('r', 8)
              .attr('fill', colors.stroke)
              .attr('stroke', 'white')
              .attr('stroke-width', 2)
              .style('cursor', 'pointer')
            
            nodeGroup
              .append('text')
              .attr('x', elkNode.width! / 2)
              .attr('y', elkNode.height! + 10)
              .attr('text-anchor', 'middle')
              .attr('dominant-baseline', 'middle')
              .attr('font-size', '12px')
              .attr('font-weight', 'bold')
              .attr('fill', 'white')
              .text('-')
              .style('cursor', 'pointer')
          }
          
          if (isActor) {
            // Actor: stick figure icon style
            const centerX = elkNode.width! / 2
            const centerY = elkNode.height! / 2
            
            // Rounded rectangle background
            nodeGroup
              .append('rect')
              .attr('width', elkNode.width!)
              .attr('height', elkNode.height!)
              .attr('rx', 12)
              .attr('ry', 12)
              .attr('fill', colors.fill)
              .attr('stroke', colors.stroke)
              .attr('stroke-width', 3)
              .attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              .style('cursor', 'pointer')
              .on('click', () => onNodeClick?.(node.id))
              .on('mouseenter', function () {
                d3.select(this).attr('stroke-width', 4).attr('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.2))')
              })
              .on('mouseleave', function () {
                d3.select(this).attr('stroke-width', 3).attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              })
          } else if (isRequirement) {
            // Requirement: document/folder style
            nodeGroup
              .append('rect')
              .attr('width', elkNode.width!)
              .attr('height', elkNode.height!)
              .attr('rx', 4)
              .attr('ry', 4)
              .attr('fill', colors.fill)
              .attr('stroke', colors.stroke)
              .attr('stroke-width', 2)
              .attr('stroke-dasharray', '5,3')
              .attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              .style('cursor', 'pointer')
              .on('click', () => onNodeClick?.(node.id))
              .on('mouseenter', function () {
                d3.select(this).attr('stroke-width', 3).attr('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.2))')
              })
              .on('mouseleave', function () {
                d3.select(this).attr('stroke-width', 2).attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              })
          } else {
            // Component/Function: Capella-style with header and ports
            const isComponent = node.type === 'component'
            const nodeWidth = elkNode.width!
            const nodeHeight = elkNode.height!
            const headerHeight = ARCVIZ_CONFIG.node.headerHeight
            
            // Main body rectangle (white)
            nodeGroup
              .append('rect')
              .attr('width', nodeWidth)
              .attr('height', nodeHeight)
              .attr('rx', ARCVIZ_CONFIG.node.borderRadius)
              .attr('ry', ARCVIZ_CONFIG.node.borderRadius)
              .attr('fill', 'white')
              .attr('stroke', colors.stroke)
              .attr('stroke-width', 2)
              .attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              .style('cursor', 'pointer')
              .on('click', () => onNodeClick?.(node.id))
              .on('mouseenter', function () {
                d3.select(this).attr('stroke-width', 3).attr('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.2))')
              })
              .on('mouseleave', function () {
                d3.select(this).attr('stroke-width', 2).attr('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.1))')
              })
            
            // Capella-style header (blue semi-transparent)
            if (isComponent) {
              nodeGroup
                .append('rect')
                .attr('width', nodeWidth)
                .attr('height', headerHeight)
                .attr('rx', ARCVIZ_CONFIG.node.borderRadius)
                .attr('ry', ARCVIZ_CONFIG.node.borderRadius)
                .attr('fill', colors.stroke)
                .attr('fill-opacity', 0.1)
                .style('pointer-events', 'none')
              
              // Header bottom border
              nodeGroup
                .append('line')
                .attr('x1', 0)
                .attr('y1', headerHeight)
                .attr('x2', nodeWidth)
                .attr('y2', headerHeight)
                .attr('stroke', colors.stroke)
                .attr('stroke-width', 1)
                .attr('stroke-opacity', 0.3)
            }
            
            // Add ports for components (IN on left, OUT on right)
            if (isComponent && node.children) {
              const portSize = ARCVIZ_CONFIG.port.size
              const portSpacing = ARCVIZ_CONFIG.port.spacing
              const functionCount = node.children.length
              
              // IN ports (left side - green)
              const inPortCount = Math.min(functionCount, 3) // Max 3 ports for demo
              for (let i = 0; i < inPortCount; i++) {
                const portY = headerHeight + 20 + i * portSpacing
                
                // Port square
                nodeGroup
                  .append('rect')
                  .attr('x', -portSize / 2)
                  .attr('y', portY - portSize / 2)
                  .attr('width', portSize)
                  .attr('height', portSize)
                  .attr('rx', ARCVIZ_CONFIG.port.borderRadius)
                  .attr('fill', ARCVIZ_CONFIG.port.colors.inFill)
                  .attr('stroke', ARCVIZ_CONFIG.port.colors.inStroke)
                  .attr('stroke-width', 2)
                
                // Port label
                nodeGroup
                  .append('text')
                  .attr('x', portSize + 4)
                  .attr('y', portY + 4)
                  .attr('font-size', `${ARCVIZ_CONFIG.label.portName.fontSize}px`)
                  .attr('font-weight', ARCVIZ_CONFIG.label.portName.fontWeight)
                  .attr('fill', ARCVIZ_CONFIG.label.portName.color)
                  .text(`IN_${i + 1}`)
              }
              
              // OUT ports (right side - orange)
              const outPortCount = Math.min(functionCount, 3)
              for (let i = 0; i < outPortCount; i++) {
                const portY = headerHeight + 20 + i * portSpacing
                
                // Port square
                nodeGroup
                  .append('rect')
                  .attr('x', nodeWidth - portSize / 2)
                  .attr('y', portY - portSize / 2)
                  .attr('width', portSize)
                  .attr('height', portSize)
                  .attr('rx', ARCVIZ_CONFIG.port.borderRadius)
                  .attr('fill', ARCVIZ_CONFIG.port.colors.outFill)
                  .attr('stroke', ARCVIZ_CONFIG.port.colors.outStroke)
                  .attr('stroke-width', 2)
                
                // Port label
                nodeGroup
                  .append('text')
                  .attr('x', nodeWidth - portSize - 4)
                  .attr('y', portY + 4)
                  .attr('font-size', `${ARCVIZ_CONFIG.label.portName.fontSize}px`)
                  .attr('font-weight', ARCVIZ_CONFIG.label.portName.fontWeight)
                  .attr('fill', ARCVIZ_CONFIG.label.portName.color)
                  .attr('text-anchor', 'end')
                  .text(`OUT_${i + 1}`)
              }
            }
          }

          // Node ID badge (top-left)
          nodeGroup
            .append('text')
            .attr('x', 8)
            .attr('y', 16)
            .attr('font-size', '9px')
            .attr('font-weight', '600')
            .attr('fill', colors.stroke)
            .text(node.id)

          // Node label (in header for components, center for others)
          const isComponent = node.type === 'component'
          const labelY = isComponent ? 35 : elkNode.height! / 2
          
          nodeGroup
            .append('text')
            .attr('x', elkNode.width! / 2)
            .attr('y', labelY)
            .attr('text-anchor', 'middle')
            .attr('dominant-baseline', 'middle')
            .attr('font-size', isComponent ? '14px' : '13px')
            .attr('font-weight', '700')
            .attr('fill', isComponent ? colors.stroke : '#1f2937')
            .text(node.label)
          
          // Function list inside components (Capella-style)
          if (isComponent && node.children && node.children.length > 0) {
            const headerHeight = ARCVIZ_CONFIG.node.headerHeight
            const funcStartY = headerHeight + ARCVIZ_CONFIG.functions.yOffset
            const lineHeight = ARCVIZ_CONFIG.functions.lineHeight
            const maxFunctions = 5 // Show max 5 functions
            
            node.children.slice(0, maxFunctions).forEach((child, index) => {
              const funcY = funcStartY + index * lineHeight
              
              // Function bullet point
              nodeGroup
                .append('circle')
                .attr('cx', ARCVIZ_CONFIG.functions.xOffset)
                .attr('cy', funcY - 3)
                .attr('r', 2)
                .attr('fill', colors.stroke)
              
              // Function name
              nodeGroup
                .append('text')
                .attr('x', ARCVIZ_CONFIG.functions.xOffset + 8)
                .attr('y', funcY)
                .attr('font-size', `${ARCVIZ_CONFIG.functions.fontSize}px`)
                .attr('font-weight', ARCVIZ_CONFIG.label.function.fontWeight)
                .attr('fill', ARCVIZ_CONFIG.label.function.color)
                .text(child.label.length > 25 ? child.label.substring(0, 22) + '...' : child.label)
            })
            
            // "... N more" indicator if there are more functions
            if (node.children.length > maxFunctions) {
              const moreY = funcStartY + maxFunctions * lineHeight
              nodeGroup
                .append('text')
                .attr('x', ARCVIZ_CONFIG.functions.xOffset + 8)
                .attr('y', moreY)
                .attr('font-size', `${ARCVIZ_CONFIG.functions.moreIndicatorSize}px`)
                .attr('font-style', 'italic')
                .attr('fill', '#9ca3af')
                .text(`... ${node.children.length - maxFunctions} more`)
            }
          }

          // Safety level badge (bottom-right)
          if (node.safetyLevel) {
            const badgeGroup = nodeGroup.append('g').attr('transform', `translate(${elkNode.width! - 60}, ${elkNode.height! - 18})`)

            badgeGroup
              .append('rect')
              .attr('width', 55)
              .attr('height', 14)
              .attr('rx', 4)
              .attr('fill', colors.stroke)

            badgeGroup
              .append('text')
              .attr('x', 27.5)
              .attr('y', 10)
              .attr('text-anchor', 'middle')
              .attr('font-size', '9px')
              .attr('font-weight', '700')
              .attr('fill', 'white')
              .text(node.safetyLevel)
          }

          // Tooltip
          nodeGroup.append('title').text(`${node.id}: ${node.label}\n${node.description || ''}\nType: ${node.type}${node.safetyLevel ? `\nSafety: ${node.safetyLevel}` : ''}`)
        })

        setIsLoading(false)
      } catch (err) {
        console.error('Error rendering diagram:', err)
        setError(err instanceof Error ? err.message : 'Failed to render diagram')
        setIsLoading(false)
      }
    }

    checkAndRender()
    
    return () => {
      renderingRef.current = false
    }
  }, [graphKey, diagramType, width, height])
  
  // Listen for zoom events from parent
  useEffect(() => {
    const handleZoomEvent = (event: CustomEvent) => {
      if (!zoomBehaviorRef.current) return
      
      const { svg, zoom } = zoomBehaviorRef.current
      const currentTransform = d3.zoomTransform(svg.node())
      
      switch (event.detail.action) {
        case 'in':
          svg.transition().duration(300).call(zoom.scaleBy, 1.3)
          break
        case 'out':
          svg.transition().duration(300).call(zoom.scaleBy, 0.7)
          break
        case 'fit':
          const initialScale = 0.8
          const centerX = width / 2 - (svg.node().getBBox().width * initialScale) / 2
          const centerY = 50
          svg.transition().duration(500).call(
            zoom.transform,
            d3.zoomIdentity.translate(centerX, centerY).scale(initialScale)
          )
          break
      }
    }
    
    window.addEventListener('diagram-zoom', handleZoomEvent as EventListener)
    return () => {
      window.removeEventListener('diagram-zoom', handleZoomEvent as EventListener)
    }
  }, [width, height])

  return (
    <div className="relative w-full h-full diagram-viewer">
      <svg ref={svgRef} width={width} height={height} className="bg-background" />
      
      {/* Zoom level indicator */}
      {!isLoading && !error && (
        <div className="absolute bottom-4 right-4 bg-white/90 backdrop-blur-sm rounded-lg px-3 py-2 shadow-lg border text-sm font-medium">
          🔍 {Math.round(zoomLevel * 100)}%
        </div>
      )}
      
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center bg-background/80">
          <div className="text-center">
            <Loader2 className="mx-auto h-8 w-8 animate-spin text-primary" />
            <p className="mt-2 text-sm text-muted-foreground">Generating layout...</p>
          </div>
        </div>
      )}
      
      {error && (
        <div className="absolute inset-0 flex items-center justify-center bg-background/80">
          <div className="text-center">
            <p className="text-destructive">Error rendering diagram</p>
            <p className="text-sm text-muted-foreground">{error}</p>
          </div>
        </div>
      )}
    </div>
  )
}
