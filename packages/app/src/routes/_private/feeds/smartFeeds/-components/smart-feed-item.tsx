import type { SmartFeed } from '@colette/core'
import { Button, Flex, Link, Text, css } from '@colette/ui'
import { Link as TLink } from '@tanstack/react-router'

type Props = {
  smartFeed: SmartFeed
}

export function SmartFeedItem({ smartFeed }: Props) {
  return (
    <Button asChild variant="ghost" title={smartFeed.title}>
      <Link asChild display="flex" textDecoration="none">
        <TLink
          to="/feeds/smartFeeds/$id"
          params={{
            id: smartFeed.id,
          }}
          activeProps={{
            className: css({
              bg: 'bg.muted',
            }),
          }}
        >
          <Text flexGrow={1} truncate>
            {smartFeed.title}
          </Text>
          <Flex justifyContent="center" w="3ch" flexShrink={0}>
            <Text
              as="span"
              color="fg.muted"
              fontVariantNumeric="tabular-nums"
              hidden={smartFeed.unreadCount === 0}
            >
              {smartFeed.unreadCount}
            </Text>
          </Flex>
        </TLink>
      </Link>
    </Button>
  )
}
