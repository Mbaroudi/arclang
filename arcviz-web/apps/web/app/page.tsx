import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { ArrowRight, Boxes, Sparkles, Zap, Shield, Users } from 'lucide-react'

export default function Home() {
  return (
    <div className="flex min-h-screen flex-col">
      {/* Header */}
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-16 items-center justify-between">
          <div className="flex items-center gap-2">
            <Boxes className="h-6 w-6 text-primary" />
            <span className="text-xl font-bold">ArcViz</span>
          </div>
          <nav className="hidden md:flex items-center gap-6">
            <Link href="#features" className="text-sm font-medium hover:text-primary transition-colors">
              Features
            </Link>
            <Link href="#pricing" className="text-sm font-medium hover:text-primary transition-colors">
              Pricing
            </Link>
            <Link href="/docs" className="text-sm font-medium hover:text-primary transition-colors">
              Docs
            </Link>
            <Button variant="ghost" asChild>
              <Link href="/login">Login</Link>
            </Button>
            <Button asChild>
              <Link href="/register">Get Started</Link>
            </Button>
          </nav>
        </div>
      </header>

      {/* Hero Section */}
      <section className="container flex flex-col items-center gap-6 py-24 md:py-32">
        <div className="flex max-w-[980px] flex-col items-center gap-4 text-center">
          <h1 className="text-4xl font-extrabold leading-tight tracking-tighter md:text-6xl lg:text-7xl">
            Modern MBSE for
            <br />
            <span className="text-primary">Complex Systems</span>
          </h1>
          <p className="max-w-[750px] text-lg text-muted-foreground sm:text-xl">
            AI-powered architecture design and validation. Build, visualize, and validate
            system architectures with the full Arcadia methodology.
          </p>
          <div className="flex gap-4 mt-4">
            <Button size="lg" asChild>
              <Link href="/register">
                Start Free Trial
                <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="#demo">Watch Demo</Link>
            </Button>
          </div>
        </div>

        {/* Hero Image Placeholder */}
        <div className="mt-12 w-full max-w-5xl rounded-lg border bg-muted p-8">
          <div className="aspect-video rounded-md bg-gradient-to-br from-primary/20 to-primary/5 flex items-center justify-center">
            <div className="text-center">
              <Boxes className="h-16 w-16 mx-auto mb-4 text-primary opacity-50" />
              <p className="text-muted-foreground">Interactive Architecture Preview</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="container py-24 md:py-32">
        <div className="mx-auto flex max-w-[980px] flex-col items-center gap-4 text-center">
          <h2 className="text-3xl font-bold leading-tight tracking-tighter md:text-5xl">
            Everything you need for MBSE
          </h2>
          <p className="max-w-[750px] text-lg text-muted-foreground">
            Powerful features to design, validate, and collaborate on complex system architectures
          </p>
        </div>

        <div className="mx-auto mt-16 grid max-w-5xl gap-8 md:grid-cols-2 lg:grid-cols-3">
          <FeatureCard
            icon={<Sparkles className="h-10 w-10" />}
            title="AI-Powered Assistant"
            description="Natural language to architecture. Let AI help you design, validate, and optimize your systems."
          />
          <FeatureCard
            icon={<Boxes className="h-10 w-10" />}
            title="Multi-View Architecture"
            description="Operational, System, Logical, Physical, and EPBS views. Full Arcadia methodology support."
          />
          <FeatureCard
            icon={<Zap className="h-10 w-10" />}
            title="Real-Time Collaboration"
            description="Work together with your team. Live cursors, comments, and change tracking."
          />
          <FeatureCard
            icon={<Shield className="h-10 w-10" />}
            title="Safety & Compliance"
            description="Built-in FMEA, FTA, and compliance checking for ISO 26262, DO-178C, and more."
          />
          <FeatureCard
            icon={<Users className="h-10 w-10" />}
            title="Enterprise Ready"
            description="SSO, audit logs, role-based access control, and on-premise deployment."
          />
          <FeatureCard
            icon={<ArrowRight className="h-10 w-10" />}
            title="Import & Export"
            description="Capella XML, SysML, Mermaid, PlantUML. Integrate with your existing tools."
          />
        </div>
      </section>

      {/* CTA Section */}
      <section className="border-t bg-muted/50">
        <div className="container flex flex-col items-center gap-4 py-24 text-center md:py-32">
          <h2 className="text-3xl font-bold leading-tight tracking-tighter md:text-5xl">
            Ready to transform your MBSE workflow?
          </h2>
          <p className="max-w-[600px] text-lg text-muted-foreground">
            Join systems engineers from leading aerospace, automotive, and defense companies.
          </p>
          <Button size="lg" className="mt-4" asChild>
            <Link href="/register">
              Start Free Trial
              <ArrowRight className="ml-2 h-4 w-4" />
            </Link>
          </Button>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t">
        <div className="container flex flex-col gap-4 py-10 md:flex-row md:justify-between">
          <div className="flex items-center gap-2">
            <Boxes className="h-5 w-5 text-primary" />
            <span className="font-semibold">ArcViz</span>
          </div>
          <p className="text-sm text-muted-foreground">
            © 2024 ArcViz. Built for Systems Engineers.
          </p>
        </div>
      </footer>
    </div>
  )
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }) {
  return (
    <div className="flex flex-col gap-4 rounded-lg border p-6">
      <div className="text-primary">{icon}</div>
      <div>
        <h3 className="mb-2 text-xl font-bold">{title}</h3>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  )
}
