'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { BookOpen, Search, ExternalLink } from 'lucide-react'

export function DocumentationPanel() {
  const [searchQuery, setSearchQuery] = useState('')

  const docSections = [
    {
      id: 'basics',
      title: 'ArcLang Basics',
      items: [
        {
          title: 'Model Structure',
          syntax: 'model <name> { ... }',
          description: 'Define a top-level architecture model',
          example: `model FlightControlSystem {
  metadata {
    name: "Flight Control System"
    version: "1.0.0"
  }
}`,
        },
        {
          title: 'Metadata Block',
          syntax: 'metadata { ... }',
          description: 'Define model metadata (name, version, author, etc.)',
          example: `metadata {
  name: "My System"
  version: "1.0.0"
  author: "John Doe"
  safety_standard: "ISO 26262"
}`,
        },
      ],
    },
    {
      id: 'requirements',
      title: 'Requirements (Capella Style)',
      items: [
        {
          title: 'Requirement Definition',
          syntax: 'req <ID> "<name>" { ... }',
          description: 'Define requirements at different levels',
          example: `requirements stakeholder {
  req STK-001 "Safe Operations" {
    description: "System shall be safe"
    priority: Critical
    safety_level: ASIL_D
  }
}

requirements system {
  req SYS-001 "Altitude Control" {
    description: "Maintain altitude"
    traces: [STK-001]
  }
}`,
        },
        {
          title: 'Safety Levels',
          syntax: 'safety_level: <level>',
          description: 'ASIL (automotive) or DAL (aerospace)',
          example: `safety_level: ASIL_D  // Automotive
safety_level: DAL_A   // Aerospace
safety_level: ASIL_B
safety_level: QM      // Quality Managed`,
        },
      ],
    },
    {
      id: 'architecture',
      title: 'Architecture Layers',
      items: [
        {
          title: 'Operational Architecture',
          syntax: 'architecture operational { ... }',
          description: 'Operational Analysis (OA) - stakeholder viewpoint',
          example: `architecture operational {
  actor "Driver" {
    description: "Vehicle operator"
  }
  
  entity "Vehicle" {
    description: "Autonomous vehicle"
  }
}`,
        },
        {
          title: 'System Architecture',
          syntax: 'architecture system { ... }',
          description: 'System Analysis (SA) - black box system view',
          example: `architecture system {
  component "ACC System" {
    description: "Adaptive Cruise Control"
    safety_level: ASIL_D
  }
}`,
        },
        {
          title: 'Logical Architecture',
          syntax: 'architecture logical { ... }',
          description: 'Logical Architecture (LA) - logical components',
          example: `architecture logical {
  component "Controller" {
    description: "Main controller"
    safety_level: ASIL_D
    
    function "ProcessData" {
      description: "Data processing"
    }
  }
}`,
        },
        {
          title: 'Physical Architecture',
          syntax: 'architecture physical { ... }',
          description: 'Physical Architecture (PA) - hardware allocation',
          example: `architecture physical {
  node "ECU1" {
    description: "Electronic Control Unit"
    allocates: [LogicalComponent1]
  }
}`,
        },
      ],
    },
    {
      id: 'components',
      title: 'Components & Functions',
      items: [
        {
          title: 'Component Definition',
          syntax: 'component <name> { ... }',
          description: 'Define a logical or physical component',
          example: `component "SensorFusion" {
  id: LC-001
  description: "Sensor data fusion"
  safety_level: ASIL_C
  
  function "MergeData" {
    description: "Merge sensor inputs"
  }
  
  function "ValidateData" {
    description: "Validate sensor data"
  }
}`,
        },
        {
          title: 'Interfaces & Ports',
          syntax: 'provides interface <name>',
          description: 'Define component interfaces (Capella style)',
          example: `component "Sensor" {
  provides interface ISensorData {
    description: "Sensor output"
    signals: [
      "Speed: Real (m/s)",
      "Distance: Real (m)"
    ]
  }
}`,
        },
      ],
    },
    {
      id: 'connections',
      title: 'Connections & Exchanges',
      items: [
        {
          title: 'Component Connection',
          syntax: 'connect <source> -> <target>',
          description: 'Connect components via interfaces',
          example: `connect Sensor.ISensorData -> Controller
connect Controller.ICommand -> Actuator`,
        },
        {
          title: 'Functional Exchange',
          syntax: 'exchange <name> { ... }',
          description: 'Define data/control exchanges',
          example: `exchange "SensorData" {
  from: Sensor
  to: Controller
  type: data
  protocol: "CAN"
}`,
        },
      ],
    },
    {
      id: 'traceability',
      title: 'Traceability',
      items: [
        {
          title: 'Traceability Links',
          syntax: 'trace <source> -> <target>',
          description: 'Trace requirements to implementation',
          example: `traceability {
  trace STK-001 -> [SYS-001, SYS-002]
  trace SYS-001 -> [Controller, Sensor]
  trace Controller -> [ECU1]
}`,
        },
      ],
    },
  ]

  const filteredSections = searchQuery
    ? docSections.map(section => ({
        ...section,
        items: section.items.filter(
          item =>
            item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
            item.description.toLowerCase().includes(searchQuery.toLowerCase())
        ),
      })).filter(section => section.items.length > 0)
    : docSections

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <BookOpen className="h-5 w-5 text-blue-600" />
          Documentation
        </CardTitle>
        <CardDescription>
          ArcLang syntax guide - Capella/Arcadia MBSE methodology
        </CardDescription>
      </CardHeader>

      <CardContent className="flex-1 flex flex-col overflow-hidden space-y-3">
        <div className="relative">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search documentation..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-8"
          />
        </div>

        <div className="flex-1 overflow-auto">
          <Accordion type="multiple" defaultValue={['basics']} className="w-full">
            {filteredSections.map((section) => (
              <AccordionItem key={section.id} value={section.id}>
                <AccordionTrigger className="text-sm font-semibold">
                  {section.title}
                  <Badge variant="secondary" className="ml-2">
                    {section.items.length}
                  </Badge>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-4">
                    {section.items.map((item, idx) => (
                      <div key={idx} className="space-y-2 pb-4 border-b last:border-b-0">
                        <div>
                          <h4 className="text-sm font-semibold">{item.title}</h4>
                          <p className="text-xs text-muted-foreground mt-1">
                            {item.description}
                          </p>
                        </div>

                        <div className="bg-blue-50 dark:bg-blue-950 px-2 py-1 rounded text-xs font-mono">
                          {item.syntax}
                        </div>

                        <div className="text-xs">
                          <div className="text-muted-foreground font-semibold mb-1">Example:</div>
                          <pre className="bg-muted p-2 rounded overflow-x-auto">
                            <code>{item.example}</code>
                          </pre>
                        </div>
                      </div>
                    ))}
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>

        <div className="pt-2 border-t">
          <a
            href="https://github.com/your-repo/arclang/wiki"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-xs text-blue-600 hover:underline"
          >
            <ExternalLink className="h-3 w-3" />
            View full documentation
          </a>
        </div>
      </CardContent>
    </Card>
  )
}
