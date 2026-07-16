'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Sparkles, Loader2, Code, BookOpen, Shield, Lightbulb, CheckCircle, XCircle, AlertTriangle } from 'lucide-react'
import { aiApi } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import { validateArcLangSyntax, AI_GENERATION_PROMPT_PREFIX, SYNTAX_EXAMPLES } from '@/lib/arclang-schema'
import { Alert, AlertDescription } from '@/components/ui/alert'

interface AIAssistantPanelProps {
  currentCode: string
  onInsertCode: (code: string) => void
}

export function AIAssistantPanel({ currentCode, onInsertCode }: AIAssistantPanelProps) {
  const [prompt, setPrompt] = useState('')
  const [isGenerating, setIsGenerating] = useState(false)
  const [result, setResult] = useState<string>('')
  const [activeTab, setActiveTab] = useState<'generate' | 'review' | 'suggest'>('generate')
  const [syntaxValidation, setSyntaxValidation] = useState<{ valid: boolean; errors: string[] } | null>(null)
  const [enforceSyntax, setEnforceSyntax] = useState(false)
  const { toast } = useToast()

  const handleGenerateRequirement = async () => {
    if (!prompt.trim()) {
      toast({
        title: 'No prompt provided',
        description: 'Please describe what you want to generate',
        variant: 'destructive',
      })
      return
    }

    setIsGenerating(true)
    setSyntaxValidation(null)
    try {
      const enhancedPrompt = enforceSyntax 
        ? `${AI_GENERATION_PROMPT_PREFIX}\n\nUser request: ${prompt}\n\nContext (existing code):\n${currentCode || 'None'}\n\nGenerate ONLY the requirement block. Ensure perfect syntax.`
        : prompt
        
      const response = await aiApi.generateRequirement(enhancedPrompt, currentCode, enforceSyntax)
      const generatedCode = response.code || response.requirement
      
      // Validate syntax
      const validation = validateArcLangSyntax(generatedCode)
      setSyntaxValidation(validation)
      
      setResult(generatedCode)
      
      if (validation.valid) {
        toast({
          title: '✓ Perfect syntax!',
          description: 'Requirement generated with valid ArcLang syntax',
        })
      } else {
        toast({
          title: 'Syntax warnings detected',
          description: `Generated code has ${validation.errors.length} syntax issues`,
          variant: 'destructive',
        })
      }
    } catch (error: any) {
      toast({
        title: 'Generation failed',
        description: error.response?.data?.error || error.message || 'Failed to generate',
        variant: 'destructive',
      })
    } finally {
      setIsGenerating(false)
    }
  }

  const handleGenerateArchitecture = async () => {
    if (!prompt.trim()) {
      toast({
        title: 'No prompt provided',
        description: 'Please describe the system architecture',
        variant: 'destructive',
      })
      return
    }

    setIsGenerating(true)
    setSyntaxValidation(null)
    try {
      const response = await aiApi.generateRichArchitecture(prompt, undefined, 'rich')
      const generatedCode = response.code
      
      // Validate syntax
      const validation = validateArcLangSyntax(generatedCode)
      setSyntaxValidation(validation)
      
      setResult(generatedCode)
      
      if (validation.valid) {
        toast({
          title: '✓ Complete architecture generated!',
          description: 'Full Capella-style architecture with traceability',
        })
      } else {
        toast({
          title: 'Architecture generated with warnings',
          description: `${validation.errors.length} syntax issues detected`,
          variant: 'destructive',
        })
      }
    } catch (error: any) {
      toast({
        title: 'Generation failed',
        description: error.response?.data?.error || error.message || 'Failed to generate',
        variant: 'destructive',
      })
    } finally {
      setIsGenerating(false)
    }
  }

  const handleSuggestArchitecture = async () => {
    setIsGenerating(true)
    try {
      const response = await aiApi.suggestArchitecture(currentCode)
      setResult(response.suggestions || response.architecture)
      toast({
        title: 'Architecture suggestions ready',
        description: 'AI has analyzed your code',
      })
    } catch (error: any) {
      toast({
        title: 'Analysis failed',
        description: error.response?.data?.error || error.message || 'Failed to analyze',
        variant: 'destructive',
      })
    } finally {
      setIsGenerating(false)
    }
  }

  const handleReviewCode = async () => {
    if (!currentCode.trim()) {
      toast({
        title: 'No code to review',
        description: 'Please write some code first',
        variant: 'destructive',
      })
      return
    }

    setIsGenerating(true)
    try {
      const response = await aiApi.reviewCode(currentCode)
      setResult(response.review || response.feedback)
      toast({
        title: 'Code review complete',
        description: 'AI has reviewed your architecture',
      })
    } catch (error: any) {
      toast({
        title: 'Review failed',
        description: error.response?.data?.error || error.message || 'Failed to review',
        variant: 'destructive',
      })
    } finally {
      setIsGenerating(false)
    }
  }

  const handleInsert = () => {
    if (result.trim()) {
      onInsertCode(result)
      setResult('')
      setPrompt('')
      toast({
        title: 'Code inserted',
        description: 'Generated code has been added to editor',
      })
    }
  }

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Sparkles className="h-5 w-5 text-purple-600" />
          AI Assistant
        </CardTitle>
        <CardDescription>
          Generate requirements, components, or get architecture suggestions using AI
        </CardDescription>
      </CardHeader>

      <CardContent className="flex-1 flex flex-col overflow-hidden">
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as any)} className="flex-1 flex flex-col">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="generate">
              <Code className="h-3 w-3 mr-1" />
              Generate
            </TabsTrigger>
            <TabsTrigger value="suggest">
              <Lightbulb className="h-3 w-3 mr-1" />
              Suggest
            </TabsTrigger>
            <TabsTrigger value="review">
              <BookOpen className="h-3 w-3 mr-1" />
              Review
            </TabsTrigger>
          </TabsList>

          <TabsContent value="generate" className="flex-1 space-y-3">
            <div>
              <Textarea
                placeholder="Describe the system to generate...&#10;&#10;Examples:&#10;• 'Adaptive Cruise Control System'&#10;• 'Lane Keeping Assist System'&#10;• 'Emergency Braking Controller'&#10;&#10;Click 'Generate' for COMPLETE Capella architecture with actors, requirements, components, functions, and traceability!"
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                className="min-h-[120px] text-sm"
                disabled={isGenerating}
              />
            </div>

            <div className="flex items-center gap-2 p-2 bg-blue-50 dark:bg-blue-950 rounded-md border border-blue-200 dark:border-blue-800 mb-2">
              <Sparkles className="h-3 w-3 text-blue-600" />
              <span className="text-xs text-blue-900 dark:text-blue-100 flex-1">
                <strong>Generate</strong> creates complete Capella architecture • <strong>Requirement</strong> creates single requirement block
              </span>
            </div>

            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={handleGenerateRequirement}
                disabled={isGenerating}
                className="flex-1"
              >
                {isGenerating ? (
                  <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                ) : (
                  <Sparkles className="mr-2 h-3 w-3" />
                )}
                Requirement
              </Button>
              <Button
                size="sm"
                onClick={handleGenerateArchitecture}
                disabled={isGenerating}
                variant="outline"
                className="flex-1"
              >
                {isGenerating ? (
                  <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                ) : (
                  <Sparkles className="mr-2 h-3 w-3" />
                )}
                Generate
              </Button>
            </div>

            {syntaxValidation && (
              <Alert variant={syntaxValidation.valid ? 'default' : 'destructive'} className="py-2">
                <div className="flex items-start gap-2">
                  {syntaxValidation.valid ? (
                    <CheckCircle className="h-4 w-4 text-green-600 mt-0.5" />
                  ) : (
                    <AlertTriangle className="h-4 w-4 mt-0.5" />
                  )}
                  <div className="flex-1 text-xs">
                    {syntaxValidation.valid ? (
                      <div className="font-semibold text-green-700 dark:text-green-300">✓ Syntax Valid</div>
                    ) : (
                      <div>
                        <div className="font-semibold mb-1">Syntax Issues:</div>
                        <ul className="list-disc list-inside space-y-1">
                          {syntaxValidation.errors.map((err, i) => (
                            <li key={i}>{err}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              </Alert>
            )}

            {result && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Badge variant={syntaxValidation?.valid ? 'default' : 'destructive'}>
                    {syntaxValidation?.valid ? '✓ Valid Code' : '⚠ Check Syntax'}
                  </Badge>
                  <Button size="sm" variant="outline" onClick={handleInsert}>
                    Insert to Editor
                  </Button>
                </div>
                <pre className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-[200px] font-mono">
                  {result}
                </pre>
              </div>
            )}
          </TabsContent>

          <TabsContent value="suggest" className="flex-1 space-y-3">
            <div className="text-sm text-muted-foreground">
              AI will analyze your current architecture and suggest improvements, missing components, or alternative designs.
            </div>

            <Button
              onClick={handleSuggestArchitecture}
              disabled={isGenerating || !currentCode.trim()}
              className="w-full"
            >
              {isGenerating ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Analyzing...
                </>
              ) : (
                <>
                  <Lightbulb className="mr-2 h-4 w-4" />
                  Get Suggestions
                </>
              )}
            </Button>

            {result && (
              <div className="space-y-2">
                <Badge variant="secondary">AI Suggestions</Badge>
                <div className="text-sm bg-muted p-3 rounded-md overflow-auto max-h-[300px] whitespace-pre-wrap">
                  {result}
                </div>
              </div>
            )}
          </TabsContent>

          <TabsContent value="review" className="flex-1 space-y-3">
            <div className="text-sm text-muted-foreground">
              Get AI-powered code review with suggestions for best practices, Capella compliance, and safety standards.
            </div>

            <Button
              onClick={handleReviewCode}
              disabled={isGenerating || !currentCode.trim()}
              className="w-full"
            >
              {isGenerating ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Reviewing...
                </>
              ) : (
                <>
                  <BookOpen className="mr-2 h-4 w-4" />
                  Review Code
                </>
              )}
            </Button>

            {result && (
              <div className="space-y-2">
                <Badge variant="secondary">Code Review</Badge>
                <div className="text-sm bg-muted p-3 rounded-md overflow-auto max-h-[300px] whitespace-pre-wrap">
                  {result}
                </div>
              </div>
            )}
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}
