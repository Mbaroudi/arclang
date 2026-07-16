import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'

export default function DocsPage() {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <Link href="/" className="text-2xl font-bold">
            ArcViz
          </Link>
          <nav className="flex items-center gap-4">
            <Link href="/editor">
              <Button variant="ghost">Editor</Button>
            </Link>
            <Link href="/visualizer">
              <Button variant="ghost">Visualizer</Button>
            </Link>
            <Link href="/login">
              <Button variant="outline">Sign In</Button>
            </Link>
          </nav>
        </div>
      </header>

      <main className="container mx-auto px-4 py-12 max-w-4xl">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-4">Documentation</h1>
          <p className="text-xl text-muted-foreground">
            Learn how to use ArcViz for Model-Based Systems Engineering
          </p>
        </div>

        <Separator className="my-8" />

        <div className="space-y-8">
          <section>
            <h2 className="text-3xl font-bold mb-4">Getting Started</h2>
            <Card>
              <CardHeader>
                <CardTitle>What is ArcViz?</CardTitle>
                <CardDescription>An introduction to the platform</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <p>
                  ArcViz is a modern, AI-powered Model-Based Systems Engineering (MBSE) platform built for the Arcadia methodology. 
                  It provides powerful tools for creating, visualizing, and analyzing system architectures.
                </p>
                <div className="bg-muted p-4 rounded-lg">
                  <h4 className="font-semibold mb-2">Key Features:</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm">
                    <li>Monaco Editor with custom ArcLang syntax highlighting</li>
                    <li>Interactive architecture diagrams with D3.js + ELK</li>
                    <li>Multi-view architecture support (Operational, System, Logical, Physical, EPBS)</li>
                    <li>Safety level management (ASIL, DAL)</li>
                    <li>Real-time compilation and validation</li>
                    <li>Traceability and requirements management</li>
                  </ul>
                </div>
              </CardContent>
            </Card>
          </section>

          <section>
            <h2 className="text-3xl font-bold mb-4">ArcLang Language</h2>
            <Card>
              <CardHeader>
                <CardTitle>Syntax Overview</CardTitle>
                <CardDescription>Learn the ArcLang modeling language</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <h4 className="font-semibold mb-2">Architecture Layers</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm">
                    <li><code className="bg-muted px-1 py-0.5 rounded">operational_analysis</code> - Define actors, capabilities, and operational activities</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">system_analysis</code> - Specify system requirements and functions</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">logical_architecture</code> - Design logical components and interfaces</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">physical_architecture</code> - Define physical nodes and deployment</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">epbs</code> - End Product Breakdown Structure</li>
                  </ul>
                </div>
                <div className="bg-slate-900 text-slate-50 p-4 rounded-lg overflow-x-auto">
                  <pre className="text-sm">
{`logical_architecture {
  component "Flight Control System" {
    id: FCS-001
    description: "Main flight control system"
    safety_level: ASIL_D
  }

  requirement "Altitude Control" {
    id: REQ-001
    priority: HIGH
  }

  trace {
    from: FCS-001
    to: REQ-001
    type: satisfies
  }
}`}
                  </pre>
                </div>
              </CardContent>
            </Card>
          </section>

          <section>
            <h2 className="text-3xl font-bold mb-4">Editor Guide</h2>
            <div className="grid gap-4 md:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Keyboard Shortcuts</CardTitle>
                </CardHeader>
                <CardContent>
                  <dl className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <dt className="font-medium">Save File</dt>
                      <dd className="text-muted-foreground"><code className="bg-muted px-2 py-0.5 rounded">Ctrl+S</code></dd>
                    </div>
                    <div className="flex justify-between">
                      <dt className="font-medium">Format Code</dt>
                      <dd className="text-muted-foreground"><code className="bg-muted px-2 py-0.5 rounded">Shift+Alt+F</code></dd>
                    </div>
                    <div className="flex justify-between">
                      <dt className="font-medium">Find</dt>
                      <dd className="text-muted-foreground"><code className="bg-muted px-2 py-0.5 rounded">Ctrl+F</code></dd>
                    </div>
                    <div className="flex justify-between">
                      <dt className="font-medium">Replace</dt>
                      <dd className="text-muted-foreground"><code className="bg-muted px-2 py-0.5 rounded">Ctrl+H</code></dd>
                    </div>
                  </dl>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Features</CardTitle>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2 text-sm list-disc list-inside">
                    <li>Syntax highlighting for ArcLang</li>
                    <li>Auto-completion</li>
                    <li>Real-time error detection</li>
                    <li>Bracket matching</li>
                    <li>Code folding</li>
                    <li>Minimap navigation</li>
                  </ul>
                </CardContent>
              </Card>
            </div>
          </section>

          <section>
            <h2 className="text-3xl font-bold mb-4">Visualizer Guide</h2>
            <Card>
              <CardHeader>
                <CardTitle>Interactive Diagrams</CardTitle>
                <CardDescription>Understand your architecture visually</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <h4 className="font-semibold mb-2">Navigation</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm">
                    <li><strong>Zoom:</strong> Mouse wheel or zoom buttons</li>
                    <li><strong>Pan:</strong> Click and drag on empty space</li>
                    <li><strong>Select Node:</strong> Click on any component, function, or requirement</li>
                    <li><strong>View Details:</strong> Node information appears in side panel</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Color Coding</h4>
                  <div className="grid gap-2 text-sm">
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-blue-500 rounded"></div>
                      <span>Components</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-purple-500 rounded"></div>
                      <span>Functions</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-pink-500 rounded"></div>
                      <span>Requirements</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-green-500 rounded"></div>
                      <span>Interfaces</span>
                    </div>
                  </div>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Safety Levels</h4>
                  <div className="grid gap-2 text-sm">
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-red-500 rounded"></div>
                      <span>ASIL-D / DAL-A (Highest criticality)</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-orange-500 rounded"></div>
                      <span>ASIL-C / DAL-B</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-yellow-500 rounded"></div>
                      <span>ASIL-B / DAL-C</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-4 h-4 bg-green-500 rounded"></div>
                      <span>ASIL-A / DAL-D</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </section>

          <section>
            <h2 className="text-3xl font-bold mb-4">API Reference</h2>
            <Card>
              <CardHeader>
                <CardTitle>REST API</CardTitle>
                <CardDescription>Integrate with ArcViz programmatically</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm">
                  The ArcViz API is available at <code className="bg-muted px-2 py-0.5 rounded">http://localhost:4000/api</code>
                </p>
                <div className="space-y-2">
                  <h4 className="font-semibold">Endpoints</h4>
                  <ul className="space-y-1 text-sm list-disc list-inside">
                    <li><code className="bg-muted px-1 py-0.5 rounded">POST /auth/register</code> - Create account</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">POST /auth/login</code> - Sign in</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">GET /projects</code> - List projects</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">POST /projects</code> - Create project</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">GET /files/project/:id</code> - List files</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">POST /compile</code> - Compile ArcLang code</li>
                    <li><code className="bg-muted px-1 py-0.5 rounded">GET /diagrams/project/:id</code> - List diagrams</li>
                  </ul>
                </div>
                <div className="bg-muted p-4 rounded-lg text-sm">
                  <p className="font-semibold mb-2">Example: Compile Code</p>
                  <pre className="overflow-x-auto">
{`curl -X POST http://localhost:4000/api/compile \\
  -H "Authorization: Bearer <token>" \\
  -H "Content-Type: application/json" \\
  -d '{"code": "logical_architecture { ... }"}'`}
                  </pre>
                </div>
              </CardContent>
            </Card>
          </section>

          <section>
            <h2 className="text-3xl font-bold mb-4">Resources</h2>
            <div className="grid gap-4 md:grid-cols-3">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">GitHub</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground mb-4">
                    View source code and contribute
                  </p>
                  <Button variant="outline" className="w-full" asChild>
                    <a href="https://github.com/mbaroudi/arcviz-web" target="_blank">
                      View on GitHub
                    </a>
                  </Button>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Arcadia</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground mb-4">
                    Learn about the methodology
                  </p>
                  <Button variant="outline" className="w-full" asChild>
                    <a href="https://www.eclipse.org/capella/" target="_blank">
                      Eclipse Capella
                    </a>
                  </Button>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Support</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground mb-4">
                    Get help and report issues
                  </p>
                  <Button variant="outline" className="w-full" asChild>
                    <a href="https://github.com/mbaroudi/arcviz-web/issues" target="_blank">
                      Report Issue
                    </a>
                  </Button>
                </CardContent>
              </Card>
            </div>
          </section>
        </div>

        <Separator className="my-12" />

        <div className="text-center">
          <h3 className="text-2xl font-bold mb-4">Ready to get started?</h3>
          <div className="flex justify-center gap-4">
            <Link href="/register">
              <Button size="lg">Create Account</Button>
            </Link>
            <Link href="/editor">
              <Button size="lg" variant="outline">
                Try Editor
              </Button>
            </Link>
          </div>
        </div>
      </main>
    </div>
  )
}
