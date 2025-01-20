import { listFeedsOptions, listSmartFeedsOptions } from '@colette/query'
import { Button } from '@colette/react-ui/components/ui/button'
import { Dialog, DialogTrigger } from '@colette/react-ui/components/ui/dialog'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { useQuery } from '@tanstack/react-query'
import { Outlet, Link as TLink, createFileRoute } from '@tanstack/react-router'
import { History, Home, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { FeedItem } from './feeds/-components/feed-item'
import { SubscribeModal } from './feeds/-components/subscribe-modal'

export const Route = createFileRoute('/_private/feeds')({
  loader: async ({ context }) => {
    const feedOptions = listFeedsOptions(
      { filterByTags: true, 'tag[]': [] },
      context.api,
    )

    const smartFeedOptions = listSmartFeedsOptions(context.api)

    await Promise.all([
      context.queryClient.ensureQueryData(feedOptions),
      context.queryClient.ensureQueryData(smartFeedOptions),
    ])

    return {
      feedOptions,
      smartFeedOptions,
    }
  },
  component: Component,
})

function Component() {
  const { feedOptions, smartFeedOptions } = Route.useLoaderData()

  const [isOpen, setOpen] = useState(false)

  const { data: feeds } = useQuery(feedOptions)
  const { data: smartFeeds } = useQuery(smartFeedOptions)

  if (!feeds || !smartFeeds) return

  return (
    <div className="flex h-full w-full">
      <div className="h-full w-[400px] space-y-4 overflow-y-auto p-4">
        <div className="flex justify-between px-4">
          <h2 className="font-medium text-3xl">Feeds</h2>
          <Dialog open={isOpen} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button className="shrink-0" variant="outline">
                <PlusCircle />
                New
              </Button>
            </DialogTrigger>
            <SubscribeModal close={() => setOpen(false)} />
          </Dialog>
        </div>
        <div className="px4 flex flex-col items-stretch gap-1">
          <Button asChild className="gap-4" variant="ghost">
            <TLink
              to="/feeds"
              activeProps={{
                className: 'bg-muted',
              }}
              activeOptions={{
                exact: true,
              }}
            >
              <Home />
              <span className="grow truncate">All Feeds</span>
            </TLink>
          </Button>
          <Button asChild className="gap-4" variant="ghost">
            <TLink
              to="/feeds/archived"
              activeProps={{
                className: 'bg-muted',
              }}
            >
              <History />
              <span className="grow truncate">Archived</span>
            </TLink>
          </Button>
        </div>
        {feeds.data.length > 0 && (
          <>
            <Separator />
            <div>
              <div className="mb-2 flex items-center justify-between">
                <span className="grow font-semibold text-muted-foreground text-xs">
                  Feeds
                </span>
              </div>
              <div className="mt-1 h-full space-y-1 overflow-y-auto">
                {feeds.data.map((feed) => (
                  <FeedItem key={feed.id} feed={feed} />
                ))}
              </div>
            </div>
          </>
        )}
      </div>
      <Separator orientation="vertical" />
      <div className="h-screen w-full overflow-y-auto">
        <Outlet />
      </div>
    </div>
  )
}
