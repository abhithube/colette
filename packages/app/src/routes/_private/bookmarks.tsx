import { Button } from '@colette/react-ui/components/ui/button'
import { Dialog, DialogTrigger } from '@colette/react-ui/components/ui/dialog'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { Link as TLink } from '@tanstack/react-router'
import { Home, Library, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { AddBookmarkModal } from './bookmarks/-components/add-bookmark-modal'

export const Route = createFileRoute('/_private/bookmarks')({
  component: Component,
})

function Component() {
  const [isOpen, setOpen] = useState(false)

  return (
    <div className="flex h-full w-full">
      <div className="h-full w-[400px] space-y-4 overflow-y-auto py-4">
        <div className="flex justify-between px-4">
          <h2 className="font-medium text-3xl">Bookmarks</h2>
          <Dialog open={isOpen} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button className="shrink-0" variant="outline">
                <PlusCircle />
                New
              </Button>
            </DialogTrigger>
            <AddBookmarkModal close={() => setOpen(false)} />
          </Dialog>
        </div>
        <div className="flex flex-col items-stretch gap-1 px-4">
          <Button asChild className="gap-4" variant="ghost">
            <TLink
              to="/bookmarks"
              activeProps={{
                className: 'bg-muted',
              }}
              activeOptions={{
                exact: true,
              }}
            >
              <Home />
              <span className="grow truncate">All Bookmarks</span>
            </TLink>
          </Button>
          <Button asChild className="gap-4" variant="ghost">
            <a href="/stash">
              <Library />
              <span className="grow truncate">Stash</span>
            </a>
          </Button>
        </div>
        <Separator />
      </div>
      <Separator orientation="vertical" />
      <div className="h-screen w-full overflow-y-auto">
        <Outlet />
      </div>
    </div>
  )
}
