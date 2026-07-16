export async function generateWithAI(prompt: string, modelPath?: string) {
  const response = await fetch('/api/ai/generate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ prompt, modelPath })
  })

  if (!response.ok) {
    throw new Error(`AI generation failed: ${response.statusText}`)
  }

  return response.json()
}

export async function askAI(question: string, context?: any) {
  const response = await fetch('/api/ai/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question, context })
  })

  if (!response.ok) {
    throw new Error(`AI chat failed: ${response.statusText}`)
  }

  return response.json()
}

export async function generateDiagramWithAI(
  modelPath: string,
  diagramType: string,
  prompt?: string
) {
  const response = await fetch('/api/diagrams/generate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      modelPath,
      diagramType,
      aiPrompt: prompt
    })
  })

  if (!response.ok) {
    throw new Error(`Diagram generation failed: ${response.statusText}`)
  }

  return response.json()
}

export async function generateAllDiagramsWithAI(
  modelPath: string,
  outputDir?: string
) {
  const response = await fetch('/api/diagrams/generate-all', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ modelPath, outputDir })
  })

  if (!response.ok) {
    throw new Error(`Bulk diagram generation failed: ${response.statusText}`)
  }

  return response.json()
}

export interface DiagramType {
  id: string
  name: string
  description: string
}

export async function getDiagramTypes(): Promise<DiagramType[]> {
  const response = await fetch('/api/diagrams/types')

  if (!response.ok) {
    throw new Error(`Failed to fetch diagram types: ${response.statusText}`)
  }

  const data = await response.json()
  return data.types
}
