import { Icon } from '@/components/icon'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Separator } from '@/components/ui/separator'
import type { FeedEntry } from '@colette/core'
import { updateFeedEntryOptions } from '@colette/query'
import { useMutation } from '@tanstack/react-query'
import { MoreHorizontal } from 'lucide-react'
import {
  EntryAuthor,
  EntryPublished,
  EntryThumbnail,
  EntryTitle,
} from '../../-components/entry-parts'
import { Route } from '../../feeds'

type Props = {
  feedEntry: FeedEntry
}

export function FeedEntryCard({ feedEntry }: Props) {
  const context = Route.useRouteContext()

  const { mutateAsync: updateFeedEntry } = useMutation(
    updateFeedEntryOptions(
      {
        onSuccess: async () => {
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feedEntries'],
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Card className="overflow-hidden shadow-md">
      <EntryThumbnail src={feedEntry.thumbnailUrl} alt={feedEntry.title} />
      <div className="flex flex-col pb-2">
        <CardHeader>
          <EntryTitle title={feedEntry.title} link={feedEntry.link} />
          <DropdownMenu>
            <DropdownMenuTrigger>
              <Icon className="text-muted-foreground" value={MoreHorizontal} />
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-56">
              <DropdownMenuItem asChild>
                <a href={feedEntry.link} target="_blank" rel="noreferrer">
                  Open in new tab
                </a>
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() =>
                  updateFeedEntry({
                    id: feedEntry.id,
                    body: {
                      hasRead: !feedEntry.hasRead,
                    },
                  })
                }
              >
                Mark as {feedEntry.hasRead ? 'unread' : 'read'}
                <DropdownMenuShortcut>⇧⌘R</DropdownMenuShortcut>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent className="flex justify-between">
          <div className="flex h-4 space-x-2">
            <EntryAuthor author={feedEntry.author} link={feedEntry.link} />
            <Separator
              className="bg-muted-foreground/50"
              orientation="vertical"
            />
            <EntryPublished publishedAt={feedEntry.publishedAt} />
          </div>
        </CardContent>
      </div>
    </Card>
  )
}
