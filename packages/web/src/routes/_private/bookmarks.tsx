import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { SidebarLink } from '../../components/sidebar'
import { AddBookmarkModal } from './bookmarks/-components/add-bookmark-modal'

export const Route = createFileRoute('/_private/bookmarks')({
  component: Component,
})

function Component() {
  const [isBookmarkModalOpen, setBookmarkModalOpen] = useState(false)

  return (
    <div className="flex h-full w-full">
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel minSize={15} defaultSize={20} maxSize={30} collapsible>
          <div className="h-full space-y-4 overflow-y-auto py-4">
            <div className="flex items-center justify-between px-4">
              <h1 className="font-medium text-3xl">Bookmarks</h1>
              <Dialog
                open={isBookmarkModalOpen}
                onOpenChange={setBookmarkModalOpen}
              >
                <DialogTrigger asChild>
                  <Button className="space-x-2">
                    <span className="text-sm">New</span>
                    <Icon value={PlusCircle} />
                  </Button>
                </DialogTrigger>
                <AddBookmarkModal close={() => setBookmarkModalOpen(false)} />
              </Dialog>
            </div>
            <div className="space-y-1 px-4">
              <SidebarLink to="/bookmarks" activeOptions={{ exact: true }}>
                <Icon value={Home} />
                <span className="grow truncate">All Bookmarks</span>
              </SidebarLink>
              <SidebarLink to="/bookmarks/stash">
                <Icon value={History} />
                <span className="grow truncate">Stash</span>
              </SidebarLink>
            </div>
            <Separator />
          </div>
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel>
          <div className="h-screen overflow-y-auto">
            <Outlet />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
