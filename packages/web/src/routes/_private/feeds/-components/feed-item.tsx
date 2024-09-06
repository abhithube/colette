import { Favicon } from '@/components/favicon'
import { Button, Flex, Link, Text, css } from '@colette/components'
import type { Feed } from '@colette/core'
import { Link as TLink } from '@tanstack/react-router'

type Props = {
  feed: Feed
}

export function FeedItem({ feed }: Props) {
  const title = feed.title ?? feed.originalTitle

  return (
    <Button asChild variant="ghost">
      <Link asChild display="flex" textDecoration="none">
        <TLink
          to="/feeds/$id"
          params={{
            id: feed.id,
          }}
          activeProps={{
            className: css({
              bg: 'bg.muted',
            }),
          }}
        >
          <Favicon domain={new URL(feed.link).hostname} />
          <Text flexGrow={1} truncate>
            {title}
          </Text>
          <Flex justifyContent="center" w="3ch" flexShrink={0}>
            <Text
              as="span"
              color="fg.muted"
              fontVariantNumeric="tabular-nums"
              hidden={feed.unreadCount === 0}
            >
              {feed.unreadCount}
            </Text>
          </Flex>
        </TLink>
      </Link>
    </Button>
  )
}
