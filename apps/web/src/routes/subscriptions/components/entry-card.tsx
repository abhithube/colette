import type { SubscriptionEntry } from '@colette/core'
import { formatRelativeDate } from '@colette/util'
import type { FC } from 'react'
import { Favicon } from '~/components/favicon'
import { Thumbnail } from '~/components/thumbnail'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { Separator } from '~/components/ui/separator'

export const EntryCard: FC<{ entry: SubscriptionEntry }> = (props) => {
  const entry = props.entry.entry

  return (
    <Card className="flex h-[160px] overflow-hidden">
      <Thumbnail src={entry.thumbnailUrl ?? undefined} alt={entry.title} />
      <div className="flex grow flex-col gap-0">
        <CardHeader className="py-0 pt-4">
          <CardTitle title={entry.title}>{entry.title}</CardTitle>
        </CardHeader>
        <CardContent className="pt-2 pb-4">
          {entry.description ? (
            <p className="line-clamp-2">{entry.description}</p>
          ) : (
            <p>No description.</p>
          )}
        </CardContent>
        <CardFooter className="justify-between py-0 pb-4">
          <div className="flex h-4 items-center gap-2 text-sm font-medium">
            <Favicon url={entry.link} />
            {entry.author && <span className="truncate">{entry.author}</span>}
            <Separator orientation="vertical" />
            <span title={new Date(entry.publishedAt).toString()}>
              {formatRelativeDate(entry.publishedAt)}
            </span>
          </div>
        </CardFooter>
      </div>
    </Card>
  )
}
