import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { listFeedsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { SidebarLink } from '../../components/sidebar'
import { FeedItem } from './feeds/-components/feed-item'
import { SubscribeModal } from './feeds/-components/subscribe-modal'

export const Route = createFileRoute('/_private/feeds')({
  loader: async ({ context }) => {
    const options = listFeedsOptions(
      { filterByTags: true, 'tag[]': [] },
      context.profile.id,
      context.api,
    )

    await context.queryClient.ensureQueryData(options)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const [isFeedModalOpen, setFeedModalOpen] = useState(false)

  const { data: feeds } = useQuery(options)

  if (!feeds) return

  return (
    <div className="flex h-full w-full">
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel minSize={15} defaultSize={20} maxSize={30} collapsible>
          <div className="h-full space-y-4 overflow-y-auto py-4">
            <div className="flex items-center justify-between px-4">
              <h1 className="font-medium text-3xl">Feeds</h1>
              <Dialog open={isFeedModalOpen} onOpenChange={setFeedModalOpen}>
                <DialogTrigger asChild>
                  <Button className="space-x-2">
                    <Icon value={PlusCircle} />
                    <span className="text-sm">New</span>
                  </Button>
                </DialogTrigger>
                <SubscribeModal close={() => setFeedModalOpen(false)} />
              </Dialog>
            </div>
            <div className="space-y-1 px-4">
              <SidebarLink to="/feeds" activeOptions={{ exact: true }}>
                <Icon value={Home} />
                <span className="grow truncate">All Feeds</span>
              </SidebarLink>
              <SidebarLink to="/feeds/archived">
                <Icon value={History} />
                <span className="grow truncate">Archived</span>
              </SidebarLink>
            </div>
            <Separator />
            {feeds.data.length > 0 && (
              <>
                <div>
                  <div className="flex h-8 items-center justify-between px-4">
                    <span className="grow font-semibold text-muted-foreground text-xs">
                      Stash
                    </span>
                  </div>
                  <div className="mt-1 h-full space-y-1 overflow-y-auto px-4">
                    {feeds.data.map((feed) => (
                      <FeedItem key={feed.id} feed={feed} />
                    ))}
                  </div>
                </div>
                <Separator />
              </>
            )}
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
