import Anthropic from '@anthropic-ai/sdk'

// Initialize Anthropic client (Claude)
const anthropic = process.env.ANTHROPIC_API_KEY 
  ? new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY })
  : null

// Alternative: OpenAI
// import OpenAI from 'openai'
// const openai = process.env.OPENAI_API_KEY 
//   ? new OpenAI({ apiKey: process.env.OPENAI_API_KEY })
//   : null

export interface AIGenerationOptions {
  systemPrompt: string
  userPrompt: string
  maxTokens?: number
  temperature?: number
}

/**
 * Generate code using Claude AI (Anthropic)
 */
export async function generateWithClaude(options: AIGenerationOptions): Promise<string> {
  if (!anthropic) {
    throw new Error('ANTHROPIC_API_KEY not configured. Please set it in .env file.')
  }

  const { systemPrompt, userPrompt, maxTokens = 8192, temperature = 0.3 } = options

  try {
    const response = await anthropic.messages.create({
      model: 'claude-sonnet-4-20250514', // Claude Sonnet 4.5
      max_tokens: maxTokens,
      temperature, // Low temperature for consistent syntax
      system: systemPrompt,
      messages: [
        {
          role: 'user',
          content: userPrompt,
        },
      ],
    })

    // Extract text from response
    const content = response.content[0]
    if (content.type === 'text') {
      return content.text
    }

    throw new Error('Unexpected response format from Claude')
  } catch (error: any) {
    console.error('Claude API error:', error)
    throw new Error(`AI generation failed: ${error.message}`)
  }
}

/**
 * Generate code using OpenAI (GPT-4)
 * Uncomment if you want to use OpenAI instead of Claude
 */
// export async function generateWithOpenAI(options: AIGenerationOptions): Promise<string> {
//   if (!openai) {
//     throw new Error('OPENAI_API_KEY not configured. Please set it in .env file.')
//   }
//
//   const { systemPrompt, userPrompt, maxTokens = 8192, temperature = 0.3 } = options
//
//   try {
//     const response = await openai.chat.completions.create({
//       model: 'gpt-4-turbo-preview',
//       max_tokens: maxTokens,
//       temperature,
//       messages: [
//         { role: 'system', content: systemPrompt },
//         { role: 'user', content: userPrompt },
//       ],
//     })
//
//     const content = response.choices[0]?.message?.content
//     if (!content) {
//       throw new Error('Empty response from OpenAI')
//     }
//
//     return content
//   } catch (error: any) {
//     console.error('OpenAI API error:', error)
//     throw new Error(`AI generation failed: ${error.message}`)
//   }
// }

/**
 * Main AI generation function (uses Claude by default)
 */
export async function generateWithAI(
  userPrompt: string,
  systemPrompt: string,
  options?: { maxTokens?: number; temperature?: number }
): Promise<string> {
  // Use Claude
  return generateWithClaude({
    systemPrompt,
    userPrompt,
    maxTokens: options?.maxTokens,
    temperature: options?.temperature,
  })

  // Or use OpenAI instead:
  // return generateWithOpenAI({
  //   systemPrompt,
  //   userPrompt,
  //   maxTokens: options?.maxTokens,
  //   temperature: options?.temperature,
  // })
}

/**
 * Extract code block from AI response (removes markdown fences if present)
 */
export function extractCodeBlock(response: string): string {
  // Remove ```arclang or ``` markdown fences
  const codeBlockMatch = response.match(/```(?:arclang)?\n([\s\S]*?)\n```/)
  if (codeBlockMatch) {
    return codeBlockMatch[1].trim()
  }

  // If no code block, return as-is (AI might return pure code)
  return response.trim()
}

/**
 * Retry generation with error feedback if validation fails
 */
export async function generateWithRetry(
  userPrompt: string,
  systemPrompt: string,
  validator: (code: string) => Promise<{ valid: boolean; errors: string[] }>,
  maxRetries: number = 3
): Promise<{ code: string; attempts: number }> {
  let lastErrors: string[] = []

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    // Enhance prompt with previous errors
    const enhancedPrompt =
      attempt === 1
        ? userPrompt
        : `${userPrompt}

IMPORTANT: Previous attempt failed with these errors:
${lastErrors.map((e, i) => `${i + 1}. ${e}`).join('\n')}

Please fix these EXACT errors and regenerate valid ArcLang code.`

    // Generate code
    const rawCode = await generateWithAI(enhancedPrompt, systemPrompt, {
      temperature: 0.2, // Even lower temperature for retries
    })

    const code = extractCodeBlock(rawCode)

    // Validate
    const validation = await validator(code)

    if (validation.valid) {
      return { code, attempts: attempt }
    }

    lastErrors = validation.errors
  }

  throw new Error(
    `Failed to generate valid code after ${maxRetries} attempts. Last errors:\n${lastErrors.join('\n')}`
  )
}
