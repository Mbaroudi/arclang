'use client'

import { useEffect, useRef } from 'react'
import Editor, { Monaco } from '@monaco-editor/react'
import { editor } from 'monaco-editor'
import { arclangLanguage, arclangTheme, arclangConfiguration } from '@/lib/arclang-syntax'
import { Loader2 } from 'lucide-react'

interface MonacoEditorProps {
  value: string
  onChange: (value: string | undefined) => void
  onValidate?: (markers: editor.IMarker[]) => void
  readOnly?: boolean
  height?: string
}

export function MonacoEditor({ value, onChange, onValidate, readOnly = false, height = '100%' }: MonacoEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<Monaco | null>(null)

  function handleEditorDidMount(editor: editor.IStandaloneCodeEditor, monaco: Monaco) {
    editorRef.current = editor
    monacoRef.current = monaco

    // Register ArcLang language
    monaco.languages.register({ id: 'arclang' })

    // Set language configuration
    monaco.languages.setLanguageConfiguration('arclang', arclangConfiguration as any)

    // Set tokens provider
    monaco.languages.setMonarchTokensProvider('arclang', arclangLanguage as any)

    // Define theme
    monaco.editor.defineTheme('arclang-dark', arclangTheme)
    monaco.editor.setTheme('arclang-dark')

    // Add validation
    monaco.editor.onDidChangeMarkers((uris) => {
      const markers = monaco.editor.getModelMarkers({})
      if (onValidate) {
        onValidate(markers)
      }
    })

    // Add keyboard shortcuts
    editor.addAction({
      id: 'save-file',
      label: 'Save File',
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
      run: () => {
        // Trigger save action
        console.log('Save triggered')
      },
    })

    editor.addAction({
      id: 'format-document',
      label: 'Format Document',
      keybindings: [monaco.KeyMod.Shift | monaco.KeyMod.Alt | monaco.KeyCode.KeyF],
      run: (editor) => {
        editor.getAction('editor.action.formatDocument')?.run()
      },
    })
  }

  return (
    <Editor
      height={height}
      defaultLanguage="arclang"
      language="arclang"
      theme="arclang-dark"
      value={value}
      onChange={onChange}
      onMount={handleEditorDidMount}
      loading={
        <div className="flex h-full items-center justify-center">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      }
      options={{
        readOnly,
        fontSize: 14,
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
        fontLigatures: true,
        minimap: { enabled: true },
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 4,
        insertSpaces: true,
        wordWrap: 'on',
        lineNumbers: 'on',
        renderLineHighlight: 'all',
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
        smoothScrolling: true,
        bracketPairColorization: {
          enabled: true,
        },
        guides: {
          bracketPairs: true,
          indentation: true,
        },
        suggest: {
          showKeywords: true,
          showSnippets: true,
        },
        quickSuggestions: {
          other: true,
          comments: false,
          strings: false,
        },
      }}
    />
  )
}
