import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { listFeedsOptions, listTagsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, Plus, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { SidebarLink } from '../../components/sidebar'
import { AddTagModal } from './-components/add-tag-modal'
import { TagItem } from './-components/tag-item'
import { FeedItem } from './feeds/-components/feed-item'
import { SubscribeModal } from './feeds/-components/subscribe-modal'

export const Route = createFileRoute('/_private/feeds')({
  loader: async ({ context }) => {
    const feedOptions = listFeedsOptions(
      { filterByTags: true, 'tag[]': [] },
      context.profile.id,
      context.api,
    )
    const tagOptions = listTagsOptions(
      { tagType: 'bookmarks' },
      context.profile.id,
      context.api,
    )

    await Promise.all([
      context.queryClient.ensureQueryData(feedOptions),
      context.queryClient.ensureQueryData(tagOptions),
    ])

    return {
      feedOptions,
      tagOptions,
    }
  },
  component: Component,
})

function Component() {
  const { feedOptions, tagOptions } = Route.useLoaderData()

  const [isFeedModalOpen, setFeedModalOpen] = useState(false)
  const [isTagModalOpen, setTagModalOpen] = useState(false)

  const { data: feeds } = useQuery(feedOptions)
  const { data: tags } = useQuery(tagOptions)

  if (!feeds || !tags) return

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
            <div>
              <div className="flex h-8 items-center justify-between px-4">
                <span className="grow font-semibold text-muted-foreground text-xs">
                  Tags
                </span>
                <Dialog open={isTagModalOpen} onOpenChange={setTagModalOpen}>
                  <DialogTrigger asChild>
                    <Button className="h-8 w-8 justify-center" variant="ghost">
                      <Icon value={Plus} />
                    </Button>
                  </DialogTrigger>
                  <AddTagModal close={() => setTagModalOpen(false)} />
                </Dialog>
              </div>
              <div className="mt-1 space-y-1 px-4">
                {tags.data.length > 0 ? (
                  <>
                    {tags.data.map((tag) => (
                      <TagItem key={tag.id} tag={tag} type="feed" />
                    ))}
                  </>
                ) : (
                  <div className="font-light text-sm">
                    You have not created any tags yet. Click + to add one.
                  </div>
                )}
              </div>
            </div>
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
