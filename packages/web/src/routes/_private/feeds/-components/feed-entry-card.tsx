import { Favicon } from '@/components/favicon'
import { formatRelativeDate } from '@/lib/utils'
import type { FeedEntry } from '@colette/core'
import { updateFeedEntryOptions } from '@colette/query'
import {
  Button,
  Card,
  Checkbox,
  Divider,
  HStack,
  Icon,
  Link,
  Text,
  VStack,
  css,
} from '@colette/ui'
import { useMutation } from '@tanstack/react-query'
import { ExternalLink } from 'lucide-react'
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
    <Card.Root h={160} flexDir="unset">
      <img
        className={css({
          aspectRatio: 16 / 9,
          bg: 'bg.default',
          objectFit: 'cover',
        })}
        src={
          feedEntry.thumbnailUrl ?? 'https://placehold.co/320x180/black/black'
        }
        alt={feedEntry.title}
        loading="lazy"
      />
      <VStack alignItems="unset" gap={0} flex={1}>
        <Card.Header py={0} pt={4}>
          <Card.Title lineClamp={1} title={feedEntry.title}>
            {feedEntry.title}
          </Card.Title>
        </Card.Header>
        <Card.Body pt={2} pb={4}>
          {feedEntry.description ? (
            <Text
              lineClamp={2}
              wordBreak="break-word"
              title={feedEntry.description}
            >
              {feedEntry.description}
            </Text>
          ) : (
            <Text>No description.</Text>
          )}
        </Card.Body>
        <Card.Footer justifyContent="space-between" py={0} pb={4}>
          <HStack gap={2} h={4} fontSize="sm" fontWeight="semibold">
            <HStack gap={2}>
              <Favicon domain={new URL(feedEntry.link).hostname} />
              {feedEntry.author && (
                <Text as="span" truncate title={feedEntry.author}>
                  {feedEntry.author}
                </Text>
              )}
            </HStack>
            <Divider orientation="vertical" />
            <Text as="span" title={new Date(feedEntry.publishedAt).toString()}>
              {formatRelativeDate(feedEntry.publishedAt)}
            </Text>
          </HStack>
          <HStack>
            <Button asChild variant="ghost" title="Open in new tab">
              <Link href={feedEntry.link} target="_blank">
                <Icon color="fg.muted">
                  <ExternalLink />
                </Icon>
              </Link>
            </Button>
            <Checkbox
              defaultChecked={feedEntry.hasRead}
              title={feedEntry.hasRead ? 'Mark as unread' : 'Mark as read'}
              onCheckedChange={(e) => {
                if (typeof e.checked === 'boolean') {
                  updateFeedEntry({
                    id: feedEntry.id,
                    body: {
                      hasRead: e.checked,
                    },
                  })
                }
              }}
            />
          </HStack>
        </Card.Footer>
      </VStack>
    </Card.Root>
  )
}
