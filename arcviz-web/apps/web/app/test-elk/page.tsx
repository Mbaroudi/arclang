'use client'

import { useEffect, useState } from 'react'
import { layoutGraph } from '@/lib/elk/elk-layout'

export default function TestElkPage() {
  const [result, setResult] = useState<string>('Testing...')

  useEffect(() => {
    async function test() {
      try {
        console.log('Starting ELK test...')
        
        const testGraph = {
          nodes: [
            { id: 'A', label: 'Node A', type: 'component' as const, width: 120, height: 60 },
            { id: 'B', label: 'Node B', type: 'component' as const, width: 120, height: 60 },
          ],
          edges: [
            { id: 'e1', source: 'A', target: 'B', type: 'data' as const },
          ],
        }
        
        console.log('Input graph:', testGraph)
        
        const layouted = await layoutGraph(testGraph)
        
        console.log('Layout result:', layouted)
        setResult(`✓ SUCCESS: Layout completed with ${layouted.children?.length} nodes`)
        
      } catch (error: any) {
        console.error('ELK test failed:', error)
        setResult(`✗ FAILED: ${error.message}`)
      }
    }
    
    test()
  }, [])

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold mb-4">ELK Layout Test</h1>
      <p className="text-lg">{result}</p>
      <p className="text-sm text-gray-600 mt-4">Check browser console for details</p>
    </div>
  )
}
