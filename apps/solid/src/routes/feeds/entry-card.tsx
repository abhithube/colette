import type { FeedEntry } from '@colette/core'
import type { Component } from 'solid-js'
import { Favicon } from '~/components/favicon'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { Separator } from '~/components/ui/separator'
import { formatRelativeDate } from '~/lib/utils'

export const EntryCard: Component<{ entry: FeedEntry }> = (props) => {
  return (
    <Card class="flex h-[160px] overflow-hidden">
      <img
        class="aspect-video object-cover"
        src={
          props.entry.thumbnailUrl ?? 'https://placehold.co/320x180/black/black'
        }
        alt={props.entry.title}
        loading="lazy"
      />
      <div class="flex grow flex-col gap-0">
        <CardHeader class="py-0 pt-4">
          <CardTitle title={props.entry.title}>{props.entry.title}</CardTitle>
        </CardHeader>
        <CardContent class="pb-4 pt-2">
          {props.entry.description ? (
            <p class="line-clamp-2">{props.entry.description}</p>
          ) : (
            <p>No description.</p>
          )}
        </CardContent>
        <CardFooter class="justify-between py-0 pb-4">
          <div class="flex h-4 items-center gap-2 text-sm font-medium">
            <Favicon url={props.entry.link} />
            {props.entry.author && (
              <span class="truncate">{props.entry.author}</span>
            )}
            <Separator orientation="vertical" />
            <span title={new Date(props.entry.publishedAt).toString()}>
              {formatRelativeDate(props.entry.publishedAt)}
            </span>
          </div>
        </CardFooter>
      </div>
    </Card>
  )
}
