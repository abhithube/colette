import { Favicon } from '@/components/favicon'
import { formatRelativeDate } from '@/lib/utils'
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
} from '@colette/components'
import type { FeedEntry } from '@colette/core'
import { updateFeedEntryOptions } from '@colette/query'
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
      <VStack alignItems="unset" gap={0}>
        <Card.Header py={0} pt={4}>
          <Card.Title lineClamp={1}>{feedEntry.title}</Card.Title>
        </Card.Header>
        <Card.Body pt={2} pb={4}>
          <Text lineClamp={2}>{feedEntry.description}</Text>
        </Card.Body>
        <Card.Footer justifyContent="space-between" py={0} pb={4}>
          <HStack gap={2} h={4} fontSize="sm" fontWeight="semibold">
            <HStack gap={2}>
              <Favicon domain={new URL(feedEntry.link).hostname} />
              <Text as="span" truncate>
                {feedEntry.author ?? 'Anonymous'}
              </Text>
            </HStack>
            <Divider orientation="vertical" />
            <Text as="span">{formatRelativeDate(feedEntry.publishedAt)}</Text>
          </HStack>
          <HStack>
            <Button asChild variant="ghost">
              <Link href={feedEntry.link} target="_blank">
                <Icon color="fg.muted">
                  <ExternalLink />
                </Icon>
              </Link>
            </Button>
            <Checkbox
              defaultChecked={feedEntry.hasRead}
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
