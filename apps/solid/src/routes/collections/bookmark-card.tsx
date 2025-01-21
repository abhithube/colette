import type { Bookmark } from '@colette/core'
import type { Component } from 'solid-js'
import { Favicon } from '~/components/favicon'
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card'
import { Separator } from '~/components/ui/separator'
import { formatRelativeDate } from '~/lib/utils'

export const BookmarkCard: Component<{ bookmark: Bookmark }> = (props) => {
  return (
    <Card class="overflow-hidden">
      <img
        class="aspect-video object-cover"
        src={
          props.bookmark.thumbnailUrl ??
          'https://placehold.co/320x180/black/black'
        }
        alt={props.bookmark.title}
        loading="lazy"
      />
      <CardHeader>
        <CardTitle title={props.bookmark.title}>
          {props.bookmark.title}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="flex h-4 items-center gap-2 text-sm font-medium">
          <Favicon url={props.bookmark.link} />
          {props.bookmark.author && (
            <span class="truncate">{props.bookmark.author}</span>
          )}
          {props.bookmark.publishedAt && (
            <>
              <Separator orientation="vertical" />
              <span title={new Date(props.bookmark.publishedAt).toString()}>
                {formatRelativeDate(props.bookmark.publishedAt)}
              </span>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
