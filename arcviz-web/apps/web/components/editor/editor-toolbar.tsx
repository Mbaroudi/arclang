'use client'

import { Button } from '@/components/ui/button'
import {
  Save,
  Download,
  Upload,
  Play,
  Eye,
  Settings,
  FileText,
  Sparkles,
  Share2,
  MoreVertical,
} from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

interface EditorToolbarProps {
  onSave: () => void
  onCompile: () => void
  onVisualize: () => void
  onExport: () => void
  onImport: () => void
  onAIAssist: () => void
  onShare?: () => void
  isCompiling?: boolean
  isSaving?: boolean
}

export function EditorToolbar({
  onSave,
  onCompile,
  onVisualize,
  onExport,
  onImport,
  onAIAssist,
  onShare,
  isCompiling = false,
  isSaving = false,
}: EditorToolbarProps) {
  return (
    <div className="flex items-center justify-between border-b bg-background px-4 py-2">
      <div className="flex items-center gap-2">
        <Button onClick={onSave} disabled={isSaving} size="sm" variant="default">
          <Save className="mr-2 h-4 w-4" />
          {isSaving ? 'Saving...' : 'Save'}
        </Button>

        <Button onClick={onCompile} disabled={isCompiling} size="sm" variant="secondary">
          <Play className="mr-2 h-4 w-4" />
          {isCompiling ? 'Compiling...' : 'Compile'}
        </Button>

        <Button onClick={onVisualize} size="sm" variant="outline">
          <Eye className="mr-2 h-4 w-4" />
          Visualize
        </Button>

        <div className="mx-2 h-6 w-px bg-border" />

        <Button onClick={onAIAssist} size="sm" variant="outline" className="text-primary">
          <Sparkles className="mr-2 h-4 w-4" />
          AI Assist
        </Button>
      </div>

      <div className="flex items-center gap-2">
        <Button size="sm" variant="ghost" onClick={onShare}>
          <Share2 className="h-4 w-4" />
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button size="sm" variant="ghost">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-48">
            <DropdownMenuItem onClick={onImport}>
              <Upload className="mr-2 h-4 w-4" />
              Import File
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onExport}>
              <Download className="mr-2 h-4 w-4" />
              Export
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>
              <FileText className="mr-2 h-4 w-4" />
              View Documentation
            </DropdownMenuItem>
            <DropdownMenuItem>
              <Settings className="mr-2 h-4 w-4" />
              Editor Settings
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
