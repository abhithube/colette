import { Thumbnail } from '../../../components/thumbnail'
import { SubscriptionEntryDetails } from '@colette/core/types'
import { Link } from '@colette/router'
import { Card, Separator, Favicon, Menu, Button } from '@colette/ui'
import { formatRelativeDate } from '@colette/util'
import {
  ArrowRight,
  Clipboard,
  ExternalLink,
  MoreHorizontal,
  Square,
  SquareCheck,
} from 'lucide-react'

export const EntryCard = (props: { details: SubscriptionEntryDetails }) => {
  const entry = props.details.feedEntry!

  return (
    <Card.Root className="overflow-hidden pt-0">
      <Thumbnail src={entry.thumbnailUrl ?? undefined} alt={entry.title} />
      <Card.Header>
        <Card.Title className="line-clamp-1 leading-tight" title={entry.title}>
          {entry.title}
        </Card.Title>
        <Card.Description className="line-clamp-2">
          {entry.description ?? 'No description.'}
        </Card.Description>
      </Card.Header>
      <Card.Footer className="justify-between">
        <div className="flex h-4 items-center gap-2 text-sm font-medium">
          <Favicon src={entry.link} />
          {entry.author && <span className="truncate">{entry.author}</span>}
          <Separator orientation="vertical" />
          <span title={new Date(entry.publishedAt).toString()}>
            {formatRelativeDate(entry.publishedAt)}
          </span>
        </div>
        <Menu.Root defaultHighlightedValue="open-link" lazyMount>
          <Menu.Trigger asChild>
            <Button variant="ghost" size="icon">
              <MoreHorizontal />
              <span className="sr-only">Entry actions</span>
            </Button>
          </Menu.Trigger>
          <Menu.Content>
            <Menu.Item value="open-link" asChild>
              <a href={entry.link} target="_blank" rel="noreferrer">
                <ExternalLink />
                Open link
              </a>
            </Menu.Item>
            <Menu.Item
              value="copy-link"
              onSelect={() => {
                navigator.clipboard.writeText(entry.link)
              }}
            >
              <Clipboard />
              Copy link
            </Menu.Item>
            <Menu.Separator />
            <Menu.Item
              value={
                props.details.subscriptionEntry.hasRead
                  ? 'mark-as-unread'
                  : 'mark-as-read'
              }
            >
              {props.details.subscriptionEntry.hasRead ? (
                <>
                  <Square /> Mark as unread
                </>
              ) : (
                <>
                  <SquareCheck /> Mark as read
                </>
              )}
            </Menu.Item>
            <Menu.Separator />
            <Menu.Item asChild value="view-feed">
              <Link
                to="/subscriptions/$subscriptionId"
                params={{
                  subscriptionId:
                    props.details.subscriptionEntry.subscriptionId,
                }}
              >
                <ArrowRight />
                View feed
              </Link>
            </Menu.Item>
          </Menu.Content>
        </Menu.Root>
      </Card.Footer>
    </Card.Root>
  )
}
