import type { FeedEntry } from '@colette/core'
import { Favicon } from '@colette/react-ui/components/favicon'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@colette/react-ui/components/ui/card'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { formatRelativeDate } from '../../../../lib/utils'

type Props = {
  feedEntry: FeedEntry
}

export function FeedEntryCard({ feedEntry }: Props) {
  return (
    <Card className="flex h-[160px] overflow-hidden">
      <img
        className="aspect-video object-cover"
        src={
          feedEntry.thumbnailUrl ?? 'https://placehold.co/320x180/black/black'
        }
        alt={feedEntry.title}
        loading="lazy"
      />
      <div className="flex grow flex-col gap-0">
        <CardHeader className="py-0 pt-4">
          <CardTitle title={feedEntry.title}>{feedEntry.title}</CardTitle>
        </CardHeader>
        <CardContent className="pt-2 pb-4">
          {feedEntry.description ? (
            <p className="line-clamp-2">{feedEntry.description}</p>
          ) : (
            <p>No description.</p>
          )}
        </CardContent>
        <CardFooter className="justify-between py-0 pb-4">
          <div className="flex h-4 items-center gap-2 font-medium text-sm">
            <Favicon url={feedEntry.link} />
            {feedEntry.author && (
              <span className="truncate">{feedEntry.author}</span>
            )}
            <Separator orientation="vertical" />
            <span title={new Date(feedEntry.publishedAt).toString()}>
              {formatRelativeDate(feedEntry.publishedAt)}
            </span>
          </div>
        </CardFooter>
      </div>
    </Card>
  )
}
