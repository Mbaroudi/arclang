'use client'

import { useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Share2, Copy, Check, Loader2, Lock } from 'lucide-react'
import { shareApi } from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'

interface ShareDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  code: string
}

export function ShareDialog({ open, onOpenChange, code }: ShareDialogProps) {
  const [isSharing, setIsSharing] = useState(false)
  const [shareUrl, setShareUrl] = useState('')
  const [copied, setCopied] = useState(false)
  const [withPassword, setWithPassword] = useState(false)
  const [password, setPassword] = useState('')
  const [expiresIn, setExpiresIn] = useState('7') // days
  const { toast } = useToast()

  const handleCreateShare = async () => {
    if (!code.trim()) {
      toast({
        title: 'No code to share',
        description: 'Please write some code first',
        variant: 'destructive',
      })
      return
    }

    setIsSharing(true)

    try {
      const options: any = {
        expiresIn: parseInt(expiresIn) * 24 * 60 * 60, // Convert days to seconds
      }

      if (withPassword && password) {
        options.password = password
      }

      const response = await shareApi.createShare(code, options)
      const url = `${window.location.origin}/share/${response.shareId}`
      setShareUrl(url)

      toast({
        title: 'Share link created',
        description: 'Your code is ready to share',
      })
    } catch (error: any) {
      toast({
        title: 'Failed to create share link',
        description: error.response?.data?.error || error.message || 'Failed to share',
        variant: 'destructive',
      })
    } finally {
      setIsSharing(false)
    }
  }

  const handleCopyUrl = () => {
    navigator.clipboard.writeText(shareUrl)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
    toast({
      title: 'Link copied',
      description: 'Share link copied to clipboard',
    })
  }

  const handleClose = () => {
    setShareUrl('')
    setPassword('')
    setWithPassword(false)
    setCopied(false)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Share2 className="h-5 w-5" />
            Share Architecture
          </DialogTitle>
          <DialogDescription>
            Create a secure, shareable link to your architecture model
          </DialogDescription>
        </DialogHeader>

        {!shareUrl ? (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="expires">Link expires in</Label>
              <Select value={expiresIn} onValueChange={setExpiresIn}>
                <SelectTrigger id="expires">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">1 day</SelectItem>
                  <SelectItem value="3">3 days</SelectItem>
                  <SelectItem value="7">7 days</SelectItem>
                  <SelectItem value="14">14 days</SelectItem>
                  <SelectItem value="30">30 days</SelectItem>
                  <SelectItem value="90">90 days</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between space-x-2 rounded-lg border p-4">
              <div className="flex-1 space-y-0.5">
                <Label htmlFor="password-protection" className="text-sm font-medium">
                  Password Protection
                </Label>
                <p className="text-xs text-muted-foreground">
                  Require a password to view the shared code
                </p>
              </div>
              <Switch
                id="password-protection"
                checked={withPassword}
                onCheckedChange={setWithPassword}
              />
            </div>

            {withPassword && (
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <div className="relative">
                  <Lock className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    id="password"
                    type="password"
                    placeholder="Enter password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="pl-8"
                  />
                </div>
              </div>
            )}

            <div className="flex justify-end gap-2 pt-4">
              <Button variant="outline" onClick={handleClose} disabled={isSharing}>
                Cancel
              </Button>
              <Button onClick={handleCreateShare} disabled={isSharing}>
                {isSharing ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Creating...
                  </>
                ) : (
                  <>
                    <Share2 className="mr-2 h-4 w-4" />
                    Create Share Link
                  </>
                )}
              </Button>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="rounded-lg border bg-muted/50 p-4 space-y-3">
              <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                <Check className="h-4 w-4" />
                <span className="font-medium">Share link created successfully</span>
              </div>

              <div className="space-y-2">
                <Label className="text-xs">Share URL</Label>
                <div className="flex gap-2">
                  <Input value={shareUrl} readOnly className="font-mono text-xs" />
                  <Button size="sm" variant="outline" onClick={handleCopyUrl}>
                    {copied ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>

              <div className="text-xs text-muted-foreground space-y-1">
                <p>• Link expires in {expiresIn} days</p>
                {withPassword && <p>• Password protected</p>}
                <p>• Read-only access</p>
              </div>
            </div>

            <div className="flex justify-end">
              <Button onClick={handleClose}>Done</Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
