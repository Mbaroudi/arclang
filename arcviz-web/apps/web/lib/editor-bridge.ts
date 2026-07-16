// Communication bridge entre Editor et Visualizer/Chat

export interface CodeUpdateEvent {
  code: string;
  source: 'chat' | 'editor' | 'ai';
  action: 'replace' | 'insert' | 'append' | 'prepend';
  position?: {
    line?: number;
    column?: number;
    section?: string; // e.g., "requirements", "components"
  };
}

export interface DiagramRequestEvent {
  diagramType: string;
  autoGenerate: boolean;
}

export interface CompilationRequestEvent {
  autoFix?: boolean;
}

export class EditorBridge {
  private static instance: EditorBridge;

  private constructor() {
    if (typeof window !== 'undefined') {
      this.setupListeners();
    }
  }

  static getInstance(): EditorBridge {
    if (!EditorBridge.instance) {
      EditorBridge.instance = new EditorBridge();
    }
    return EditorBridge.instance;
  }

  private setupListeners() {
    // Écoute les mises à jour de code depuis le chat
    window.addEventListener('chat:update-code', ((e: CustomEvent<CodeUpdateEvent>) => {
      console.log('[EditorBridge] Code update from chat:', e.detail);
      this.handleCodeUpdate(e.detail);
    }) as EventListener);

    // Écoute les demandes de compilation
    window.addEventListener('chat:compile', ((e: CustomEvent<CompilationRequestEvent>) => {
      console.log('[EditorBridge] Compilation request from chat:', e.detail);
      this.requestCompilation(e.detail);
    }) as EventListener);

    // Écoute les demandes de génération de diagrammes
    window.addEventListener('chat:generate-diagram', ((e: CustomEvent<DiagramRequestEvent>) => {
      console.log('[EditorBridge] Diagram request from chat:', e.detail);
      this.requestDiagram(e.detail);
    }) as EventListener);
  }

  // === FROM CHAT TO EDITOR ===

  updateCodeFromChat(update: CodeUpdateEvent) {
    window.dispatchEvent(new CustomEvent('chat:update-code', { detail: update }));
  }

  requestCompilation(request: CompilationRequestEvent = {}) {
    window.dispatchEvent(new CustomEvent('chat:compile', { detail: request }));
  }

  requestDiagram(request: DiagramRequestEvent) {
    window.dispatchEvent(new CustomEvent('chat:generate-diagram', { detail: request }));
  }

  // === FROM EDITOR TO CHAT/VISUALIZER ===

  notifyCodeChanged(code: string) {
    localStorage.setItem('arcviz_current_model', code);
    window.dispatchEvent(new CustomEvent('editor:code-changed', { 
      detail: { code, timestamp: Date.now() } 
    }));
  }

  notifyCompilationComplete(success: boolean, errors?: string[]) {
    window.dispatchEvent(new CustomEvent('editor:compilation-complete', {
      detail: { success, errors, timestamp: Date.now() }
    }));
  }

  notifyDiagramGenerated(diagramType: string, svg: string) {
    window.dispatchEvent(new CustomEvent('editor:diagram-generated', {
      detail: { diagramType, svg, timestamp: Date.now() }
    }));
  }

  // === SMART CODE INSERTION ===

  private handleCodeUpdate(update: CodeUpdateEvent) {
    const currentCode = localStorage.getItem('arcviz_current_model') || '';
    let newCode = currentCode;

    switch (update.action) {
      case 'replace':
        newCode = update.code;
        break;

      case 'insert':
        newCode = this.smartInsert(currentCode, update.code, update.position);
        break;

      case 'append':
        newCode = currentCode + '\n\n' + update.code;
        break;

      case 'prepend':
        newCode = update.code + '\n\n' + currentCode;
        break;
    }

    // Sauvegarde et notifie l'éditeur
    localStorage.setItem('arcviz_current_model', newCode);
    window.dispatchEvent(new CustomEvent('editor:reload-code', { 
      detail: { code: newCode, source: update.source } 
    }));
  }

  private smartInsert(currentCode: string, newCode: string, position?: CodeUpdateEvent['position']): string {
    if (!position) {
      return currentCode + '\n\n' + newCode;
    }

    // Insertion par section
    if (position.section) {
      return this.insertInSection(currentCode, newCode, position.section);
    }

    // Insertion par ligne/colonne
    if (position.line !== undefined) {
      const lines = currentCode.split('\n');
      lines.splice(position.line, 0, newCode);
      return lines.join('\n');
    }

    return currentCode + '\n\n' + newCode;
  }

  private insertInSection(code: string, newCode: string, section: string): string {
    const sectionPatterns: Record<string, RegExp> = {
      requirements: /requirement\s+\w+\s*{[^}]*}/g,
      components: /component\s+\w+\s*{[^}]*}/g,
      functions: /function\s+\w+\s*{[^}]*}/g,
      actors: /actor\s+\w+\s*{[^}]*}/g,
    };

    const pattern = sectionPatterns[section];
    if (!pattern) {
      return code + '\n\n' + newCode;
    }

    const matches = Array.from(code.matchAll(pattern));
    if (matches.length > 0) {
      const lastMatch = matches[matches.length - 1];
      const insertPos = lastMatch.index! + lastMatch[0].length;
      return code.slice(0, insertPos) + '\n\n' + newCode + code.slice(insertPos);
    }

    return code + '\n\n' + newCode;
  }

  // === HELPERS ===

  getCurrentCode(): string {
    return localStorage.getItem('arcviz_current_model') || '';
  }

  analyzeCodeStructure(code: string) {
    return {
      hasRequirements: /requirement\s+\w+/g.test(code),
      hasComponents: /component\s+\w+/g.test(code),
      hasFunctions: /function\s+\w+/g.test(code),
      hasActors: /actor\s+\w+/g.test(code),
      lineCount: code.split('\n').length,
    };
  }
}

// Export singleton instance
export const editorBridge = EditorBridge.getInstance();
