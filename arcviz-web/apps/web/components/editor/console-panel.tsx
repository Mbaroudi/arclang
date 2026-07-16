'use client'

import { editor } from 'monaco-editor'
import { AlertCircle, CheckCircle2, Info, XCircle } from 'lucide-react'
import { cn } from '@/lib/utils'

interface ConsolePanelProps {
  markers: editor.IMarker[]
  compilationResult?: {
    success: boolean
    message: string
    stats?: {
      requirements: number
      components: number
      functions: number
      traces: number
    }
  }
}

export function ConsolePanel({ markers, compilationResult }: ConsolePanelProps) {
  const errors = markers.filter((m) => m.severity === 8) // Error
  const warnings = markers.filter((m) => m.severity === 4) // Warning
  const infos = markers.filter((m) => m.severity === 2) // Info

  return (
    <div className="flex h-full flex-col border-t bg-muted/30">
      {/* Header */}
      <div className="flex items-center justify-between border-b bg-background px-4 py-2">
        <div className="flex items-center gap-4 text-sm">
          <span className="font-medium">Console</span>
          {errors.length > 0 && (
            <span className="flex items-center gap-1 text-destructive">
              <XCircle className="h-3.5 w-3.5" />
              {errors.length} {errors.length === 1 ? 'Error' : 'Errors'}
            </span>
          )}
          {warnings.length > 0 && (
            <span className="flex items-center gap-1 text-yellow-600">
              <AlertCircle className="h-3.5 w-3.5" />
              {warnings.length} {warnings.length === 1 ? 'Warning' : 'Warnings'}
            </span>
          )}
          {errors.length === 0 && warnings.length === 0 && (
            <span className="flex items-center gap-1 text-green-600">
              <CheckCircle2 className="h-3.5 w-3.5" />
              No Issues
            </span>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {/* Compilation Result */}
        {compilationResult && (
          <div
            className={cn(
              'mb-4 rounded-md border p-3',
              compilationResult.success
                ? 'border-green-600/20 bg-green-600/10 text-green-600'
                : 'border-destructive/20 bg-destructive/10 text-destructive'
            )}
          >
            <div className="flex items-start gap-2">
              {compilationResult.success ? (
                <CheckCircle2 className="h-5 w-5 flex-shrink-0" />
              ) : (
                <XCircle className="h-5 w-5 flex-shrink-0" />
              )}
              <div className="flex-1">
                <p className="font-medium">
                  {compilationResult.success ? 'Compilation Successful' : 'Compilation Failed'}
                </p>
                <p className="mt-1 text-sm opacity-90">{compilationResult.message}</p>
                {compilationResult.stats && compilationResult.success && (
                  <div className="mt-2 grid grid-cols-4 gap-4 text-sm">
                    <div>
                      <span className="opacity-70">Requirements:</span>{' '}
                      <span className="font-medium">{compilationResult.stats.requirements}</span>
                    </div>
                    <div>
                      <span className="opacity-70">Components:</span>{' '}
                      <span className="font-medium">{compilationResult.stats.components}</span>
                    </div>
                    <div>
                      <span className="opacity-70">Functions:</span>{' '}
                      <span className="font-medium">{compilationResult.stats.functions}</span>
                    </div>
                    <div>
                      <span className="opacity-70">Traces:</span>{' '}
                      <span className="font-medium">{compilationResult.stats.traces}</span>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Markers List */}
        {markers.length > 0 ? (
          <div className="space-y-2">
            {markers.map((marker, index) => (
              <div
                key={index}
                className={cn(
                  'flex items-start gap-3 rounded-md border p-3 text-sm',
                  marker.severity === 8 && 'border-destructive/20 bg-destructive/5',
                  marker.severity === 4 && 'border-yellow-600/20 bg-yellow-600/5',
                  marker.severity === 2 && 'border-blue-600/20 bg-blue-600/5'
                )}
              >
                <div className="flex-shrink-0">
                  {marker.severity === 8 && <XCircle className="h-4 w-4 text-destructive" />}
                  {marker.severity === 4 && <AlertCircle className="h-4 w-4 text-yellow-600" />}
                  {marker.severity === 2 && <Info className="h-4 w-4 text-blue-600" />}
                </div>
                <div className="flex-1 space-y-1">
                  <p className="font-medium">{marker.message}</p>
                  <p className="text-xs text-muted-foreground">
                    Line {marker.startLineNumber}, Column {marker.startColumn}
                  </p>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            <p className="text-sm">No messages to display</p>
          </div>
        )}
      </div>
    </div>
  )
}
