import type { Feed } from '@colette/core'
import { Favicon } from '@colette/react-ui/components/favicon'
import { Button } from '@colette/react-ui/components/ui/button'
import { cn } from '@colette/react-ui/lib/utils'
import { Link as TLink } from '@tanstack/react-router'

type Props = {
  feed: Feed
}

export function FeedItem({ feed }: Props) {
  const title = feed.title ?? feed.originalTitle

  return (
    <Button
      asChild
      className="flex items-center gap-4"
      variant="ghost"
      title={title}
    >
      <TLink
        to="/feeds/$id"
        params={{
          id: feed.id,
        }}
        activeProps={{
          className: 'bg-muted',
        }}
      >
        <Favicon url={feed.link} />
        <span className="grow truncate">{title}</span>
        <div className="flex w-[3ch] shrink-0 justify-center">
          <span
            className={cn(
              'text-muted-foreground tabular-nums',
              feed.unreadCount === 0 && 'hidden',
            )}
          >
            {feed.unreadCount}
          </span>
        </div>
      </TLink>
    </Button>
  )
}
