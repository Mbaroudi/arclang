import Anthropic from '@anthropic-ai/sdk';
import { PrismaClient } from '@prisma/client';

const anthropic = process.env.ANTHROPIC_API_KEY
  ? new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY })
  : null;

export interface ConversationMessage {
  role: string;
  content: string;
}

export interface ConversationContext {
  projectId?: string;
  diagramType?: string;
  currentModel?: any;
  recentCorrections?: any[];
  userPreferences?: Record<string, any>;
}

export interface AIResponse {
  content: string;
  generatedCode?: string;
  diagramSvg?: string;
  diagramType?: string;
  actions?: Array<{
    type: 'generate_diagram' | 'fix_code' | 'validate_code' | 'update_code' | 'insert_code' | 'replace_code' | 'compile_code';
    payload: any;
  }>;
}

export class ConversationalAIService {
  private prisma: PrismaClient;

  constructor(prisma: PrismaClient) {
    this.prisma = prisma;
  }

  async generateResponse(
    conversationHistory: ConversationMessage[],
    context: ConversationContext = {}
  ): Promise<AIResponse> {
    if (!anthropic) {
      throw new Error('ANTHROPIC_API_KEY not configured');
    }

    const systemPrompt = await this.buildSystemPrompt(context);

    const messages = conversationHistory.map((msg) => ({
      role: msg.role as 'user' | 'assistant',
      content: msg.content,
    }));

    try {
      const response = await anthropic.messages.create({
        model: 'claude-sonnet-4-20250514',
        max_tokens: 4096,
        temperature: 0.3,
        system: systemPrompt,
        messages,
      });

      const content = response.content[0];
      if (content.type !== 'text') {
        throw new Error('Unexpected response type from AI');
      }

      const generatedCode = this.extractCodeBlock(content.text);
      const diagramType = this.detectDiagramType(content.text, context);
      const actions = this.extractActions(content.text, context);

      return {
        content: content.text,
        generatedCode,
        diagramType,
        actions,
      };
    } catch (error) {
      console.error('AI generation error:', error);
      throw new Error('Failed to generate AI response');
    }
  }

  private async buildSystemPrompt(context: ConversationContext): Promise<string> {
    let prompt = `You are an expert ArcLang systems engineering assistant specializing in MBSE (Model-Based Systems Engineering) with Capella/Arcadia methodology.

You help users create, review, and refine:
- System requirements and architecture
- MBSE diagrams (Dataflow, Sequence, State Machine, Class, Component, Allocation, System Context, etc.)
- ArcLang code following ISO 26262, DO-178C, and IEC 61508 safety standards
- The 7 Arcadia dimensions: Operational Analysis, System Analysis, Logical Architecture, Physical Architecture, EPBS, Requirements Traceability, and Cross-cutting Concerns

When generating code:
- Follow ArcLang syntax rules strictly
- Use proper safety levels (ASIL-A through ASIL-D, DAL-A through DAL-D, SIL-0 through SIL-4)
- Include traceability information
- Provide clear, helpful explanations
- Consider the 7 Arcadia dimensions when designing systems
- **ALWAYS include MBSE Capella features for professional diagrams**

When responding to corrections:
- Acknowledge the issue clearly
- Explain what was wrong
- Provide the corrected version
- Learn from the mistake for future generations

IMPORTANT - Actionable Responses:
When users ask you to generate diagrams, fix code, or validate, be explicit in your response:
- Say "I will generate a [diagram-type] diagram" to trigger diagram generation
- Provide fixed code in a code block when suggesting corrections
- Say "Let me validate your code" when checking syntax

Examples:
- User: "Generate a dataflow diagram" → Response: "I will generate a dataflow diagram showing..."
- User: "Fix the syntax error" → Response: "Here's the corrected code: \`\`\`arclang ... \`\`\`"
- User: "Check my code" → Response: "Let me validate your code..."

═══════════════════════════════════════════════════════════════════════════
🎨 MBSE CAPELLA FEATURES - MANDATORY FOR ALL CODE GENERATION
═══════════════════════════════════════════════════════════════════════════

**CRITICAL**: ALL generated ArcLang code MUST include these features for proper professional diagram rendering:

1️⃣ SAFETY COLORS (ISO 26262, DO-178C, IEC 61508)
──────────────────────────────────────────────────
Add safety_level to ALL safety-critical elements:

✅ Components (Logical Architecture):
   component BrakeController "Brake Controller" {
       safety_level: ASIL_D  // Red 6px border
   }

✅ Physical Nodes (Physical Architecture):
   component BrakeECU "Brake ECU" {
       node_type: Hardware
       safety_level: ASIL_D  // Red 6px border on hardware
   }

✅ Operational Activities (Operational Analysis):
   **IMPORTANT**: Use operational_analysis "Title" { } syntax, NOT architecture operational
   
   operational_analysis "Emergency Braking Context" {
       actor "Vehicle System" {
           id: "OA-ACT-001"
           description: "Automated braking system"
           category: "System"
           safety_level: ASIL_D  // Red border on safety-critical actor
       }
       
       operational_activity "Execute Emergency Braking" {
           id: "OA-01"
           description: "Apply maximum braking"
           performed_by: "OA-ACT-001"
           safety_level: ASIL_D  // Red border on critical activity
       }
       
       operational_interaction "Collision Alert" {
           id: "OI-01"
           from: "OA-ACT-001"
           to: "OA-ACT-002"
           exchange_item_kind: EVENT  // Red dashed alert
       }
   }

✅ Sequence Messages (Safety-critical messages):
   message BrakeCommand {
       safety_level: ASIL_D  // Red thick arrow (6px)
       message_type: Synchronous
   }

✅ State Machine States (Safety states):
   state EmergencyBraking {
       safety_level: ASIL_D  // Red border on state
   }

Safety Levels:
- ASIL_D (Red 6px) - Most critical automotive
- ASIL_C (Orange 5px) - High automotive
- ASIL_B (Orange 4px) - Medium automotive
- ASIL_A (Yellow 3px) - Low automotive
- QM (Gray 2px) - Quality managed
- dal: DAL_A through DAL_E (Aerospace DO-178C)
- sil: SIL_4 through SIL_0 (Industrial IEC 61508)

2️⃣ EXCHANGE ITEM TYPES (Functional/Component Exchanges)
──────────────────────────────────────────────────────────
Add exchange_item_kind to ALL exchanges:

✅ EVENT - Signal with no data (⚡ Red dashed):
   exchange AlertExchange {
       exchange_item_kind: EVENT
       label: "CollisionAlert"
   }

✅ FLOW - Continuous stream (⟿ Cyan thick):
   exchange DataStream {
       exchange_item_kind: FLOW
       label: "ContinuousData"
   }

✅ OPERATION - Request/response (↔ Teal double):
   exchange ServiceCall {
       exchange_item_kind: OPERATION
       label: "Calculate"
   }

✅ DATA - Structured package (📦 Blue solid, DEFAULT):
   exchange DataPacket {
       exchange_item_kind: DATA
       label: "Telemetry"
   }

✅ SHARED_DATA - Repository access (🗄 Purple dotted):
   exchange ConfigAccess {
       exchange_item_kind: SHARED_DATA
       label: "ConfigDB"
   }

✅ UNSET - Generic/unknown (→ Gray thin):
   exchange GenericExchange {
       exchange_item_kind: UNSET
       label: "Unknown"
   }

3️⃣ INTERFACE NOTATION (UML/SysML Ball-and-Socket)
────────────────────────────────────────────────────
Use provides/requires for proper interface notation:

✅ Provided Interfaces (Lollipops ○ on RIGHT side):
   component DataProducer {
       provides interface IData {
           signals: ["rawData: ByteArray", "timestamp: Time"]
       }
   }

✅ Required Interfaces (Sockets ⌒ on LEFT side):
   component DataConsumer {
       requires interface IData
       requires interface IConfig {
           signals: ["mode: Enum"]
       }
   }

4️⃣ TRACEABILITY LINKS (Cross-Layer)
──────────────────────────────────────
Add explicit traceability:

✅ Allocation (Function → Component):
   component Controller {
       allocated_functions: [SF-001, SF-002]  // Orange allocation lines
   }

✅ Implementation (Component → Hardware):
   component ECU {
       implements: [LogicalComponent]
   }

✅ Traces (Requirements → Components):
   traceability {
       trace REQ-001 -> [Component1, Component2]
   }

═══════════════════════════════════════════════════════════════════════════
📋 MANDATORY CHECKLIST FOR EVERY GENERATION
═══════════════════════════════════════════════════════════════════════════

Before generating ANY ArcLang code, ensure:

☑️ Safety Colors:
   - Add safety_level to components (LA)
   - Add safety_level to physical nodes (PA)
   - Add safety_level to activities (OA)
   - Add safety_level to messages (Sequence)
   - Add safety_level to states (State Machine)

☑️ Exchange Items:
   - Add exchange_item_kind to ALL functional exchanges
   - Add exchange_item_kind to ALL component exchanges
   - Use EVENT for alerts/signals
   - Use FLOW for continuous streams
   - Use OPERATION for service calls
   - Use DATA for structured data (default)

☑️ Interface Notation:
   - Use provides interface IName for provided interfaces
   - Use requires interface IName for required interfaces
   - Define signals in interface blocks
   - Use IName (capital I prefix) naming

☑️ Traceability:
   - Add allocated_functions to components
   - Add implements to physical nodes
   - Add traces to requirements
   - Include traceability block

☑️ Complete Structure:
   - Wrap all in model { } block
   - Include metadata with version and safety_standard
   - Add requirements with safety_level
   - Multiple architecture views when appropriate
   - Traceability block at end

═══════════════════════════════════════════════════════════════════════════
⚠️ CRITICAL VALIDATION RULES
═══════════════════════════════════════════════════════════════════════════

**The following will cause diagram rendering FAILURES:**

❌ Missing exchange_item_kind → Generic gray lines instead of semantic colors
❌ Missing safety_level → No safety borders on critical components
❌ Using port instead of provides/requires → No UML ball-and-socket notation
❌ Missing allocated_functions → No allocation lines
❌ String identifiers → Parser errors
❌ Missing model wrapper → Top-level blocks rejected

**ALWAYS generate complete, professional MBSE models with ALL features enabled.**`;

    if (context.diagramType) {
      prompt += `\n\nCurrent context: Working on ${context.diagramType} diagram.`;
    }

    if ((context as any).currentCode) {
      const code = (context as any).currentCode;
      prompt += `\n\nUser's current ArcLang code:\n\`\`\`arclang\n${code.substring(0, 3000)}\n\`\`\`\n\nAnalyze this code when answering questions and provide specific suggestions based on it.`;
    }

    const userId = context.userPreferences?.userId;
    if (userId) {
      const commonErrors = await this.getCommonErrorsForUser(userId, context.diagramType);
      if (commonErrors.length > 0) {
        prompt += `\n\n**IMPORTANT - Avoid these common mistakes:**\n`;
        commonErrors.forEach((error) => {
          prompt += `- ${error.errorSignature}: ${error.solutionPattern}\n`;
        });
      }
    }

    if (context.recentCorrections && context.recentCorrections.length > 0) {
      prompt += `\n\n**Recent corrections in this conversation:**\n`;
      context.recentCorrections.forEach((correction) => {
        prompt += `- ${correction.userFeedback}\n`;
      });
    }

    return prompt;
  }

  private async getCommonErrorsForUser(
    userId: string,
    diagramType?: string
  ): Promise<any[]> {
    const errors = await this.prisma.errorPattern.findMany({
      where: {
        OR: [{ userId }, { userId: null }],
        ...(diagramType && { diagramType }),
      },
      orderBy: [{ frequency: 'desc' }, { successRate: 'asc' }],
      take: 5,
    });

    return errors;
  }

  private extractCodeBlock(text: string): string | undefined {
    const codeBlockRegex = /```(?:arclang)?\n([\s\S]*?)```/;
    const match = text.match(codeBlockRegex);
    return match ? match[1].trim() : undefined;
  }

  private detectDiagramType(text: string, context: ConversationContext): string | undefined {
    if (context.diagramType) {
      return context.diagramType;
    }

    const lowerText = text.toLowerCase();
    const diagramTypes = [
      'dataflow',
      'sequence',
      'state-machine',
      'class',
      'component',
      'physical',
      'tree',
      'capability',
      'functional-chain',
      'allocation',
      'system-context',
    ];

    for (const type of diagramTypes) {
      if (lowerText.includes(type) || lowerText.includes(type.replace('-', ' '))) {
        return type;
      }
    }

    return undefined;
  }

  async generateDiagram(
    _diagramType: string,
    _modelCode: string,
    _conversationContext: ConversationContext = {}
  ): Promise<{ svg: string; metadata: any }> {
    return {
      svg: '<svg></svg>',
      metadata: {},
    };
  }

  private extractActions(text: string, context: ConversationContext): Array<{type: string; payload: any}> {
    const actions: Array<{type: string; payload: any}> = [];
    const lowerText = text.toLowerCase();

    if (lowerText.includes('generate') && lowerText.includes('diagram')) {
      const diagramType = this.detectDiagramType(text, context);
      if (diagramType) {
        actions.push({
          type: 'generate_diagram',
          payload: { diagramType }
        });
      }
    }

    const code = this.extractCodeBlock(text);

    if (lowerText.includes('fix') || lowerText.includes('correct') || lowerText.includes('error')) {
      if (code) {
        actions.push({
          type: 'replace_code',
          payload: { code, action: 'replace' }
        });
      }
    }

    if (lowerText.includes('add') || lowerText.includes('insert') || lowerText.includes('create')) {
      if (code) {
        let section = 'end';
        if (lowerText.includes('requirement')) section = 'requirements';
        else if (lowerText.includes('component')) section = 'components';
        else if (lowerText.includes('function')) section = 'functions';
        else if (lowerText.includes('actor')) section = 'actors';

        actions.push({
          type: 'insert_code',
          payload: { code, section, action: 'insert' }
        });
      }
    }

    if (lowerText.includes('compile') || lowerText.includes('build')) {
      actions.push({
        type: 'compile_code',
        payload: { autoFix: lowerText.includes('auto') || lowerText.includes('fix') }
      });
    }

    if (lowerText.includes('validate') || lowerText.includes('check syntax')) {
      actions.push({
        type: 'validate_code',
        payload: {}
      });
    }

    return actions;
  }
}

export function createConversationalAIService(prisma: PrismaClient): ConversationalAIService {
  return new ConversationalAIService(prisma);
}
