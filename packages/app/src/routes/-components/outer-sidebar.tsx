import { Button } from '@colette/react-ui/components/ui/button'
import { Dialog, DialogTrigger } from '@colette/react-ui/components/ui/dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@colette/react-ui/components/ui/tooltip'
import { Link as TLink } from '@tanstack/react-router'
import { Bookmark, Home, Rss, Search, Settings, User } from 'lucide-react'
import { useState } from 'react'
import { SettingsModal } from './settings-modal'

export const OuterSidebar = () => {
  const [isOpen, setOpen] = useState(false)

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button asChild variant="ghost">
              <TLink
                to="/"
                activeProps={{
                  className: 'bg-muted',
                }}
              >
                <Home />
              </TLink>
            </Button>
          </TooltipTrigger>
          <TooltipContent>Home</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button asChild variant="ghost">
              <TLink
                to="/feeds"
                activeProps={{
                  className: 'bg-muted',
                }}
                activeOptions={{
                  exact: false,
                }}
              >
                <Rss />
              </TLink>
            </Button>
          </TooltipTrigger>
          <TooltipContent>Feed</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button asChild variant="ghost">
              <TLink
                to="/bookmarks"
                activeProps={{
                  className: 'bg-muted',
                }}
                activeOptions={{
                  exact: false,
                }}
              >
                <Bookmark />
              </TLink>
            </Button>
          </TooltipTrigger>
          <TooltipContent>Bookmarks</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost">
              <Search />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Search</TooltipContent>
        </Tooltip>
        <div className="flex grow" />
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost">
              <User />
            </Button>
          </TooltipTrigger>
          <TooltipContent>User</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <Dialog open={isOpen} onOpenChange={setOpen}>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <DialogTrigger asChild>
                <Button variant="ghost">
                  <Settings />
                </Button>
              </DialogTrigger>
            </TooltipTrigger>
            <TooltipContent>Settings</TooltipContent>
          </Tooltip>
        </TooltipProvider>
        <SettingsModal close={() => setOpen(false)} />
      </Dialog>
    </div>
  )
}
