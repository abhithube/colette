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
import type { Entry } from '@colette/openapi'
import { updateEntryOptions } from '@colette/query'
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
  entry: Entry
}

export function FeedEntryCard({ entry }: Props) {
  const context = Route.useRouteContext()

  const { mutateAsync: updateEntry } = useMutation(
    updateEntryOptions(
      {
        onSuccess: async () => {
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'entries'],
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Card className="overflow-hidden shadow-md">
      <EntryThumbnail src={entry.thumbnailUrl} alt={entry.title} />
      <div className="flex flex-col pb-2">
        <CardHeader>
          <EntryTitle title={entry.title} link={entry.link} />
          <DropdownMenu>
            <DropdownMenuTrigger>
              <Icon className="text-muted-foreground" value={MoreHorizontal} />
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-56">
              <DropdownMenuItem asChild>
                <a href={entry.link} target="_blank" rel="noreferrer">
                  Open in new tab
                </a>
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() =>
                  updateEntry({
                    id: entry.id,
                    body: {
                      hasRead: !entry.hasRead,
                    },
                  })
                }
              >
                Mark as {entry.hasRead ? 'unread' : 'read'}
                <DropdownMenuShortcut>⇧⌘R</DropdownMenuShortcut>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent className="flex justify-between">
          <div className="flex h-4 space-x-2">
            <EntryAuthor author={entry.author} link={entry.link} />
            <Separator
              className="bg-muted-foreground/50"
              orientation="vertical"
            />
            {entry.publishedAt && (
              <EntryPublished publishedAt={entry.publishedAt} />
            )}
          </div>
        </CardContent>
      </div>
    </Card>
  )
}
