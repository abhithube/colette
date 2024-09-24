import type { FeedEntry } from '@colette/core'
import { Box, Container, Divider, HStack, Text } from '@colette/ui'
import { useInView } from 'react-intersection-observer'
import { FeedEntryCard } from './feed-entry-card'

type Props = {
  feedEntries: FeedEntry[]
  hasMore: boolean
  loadMore?: () => void
}

export function FeedEntryGrid({ feedEntries, hasMore, loadMore }: Props) {
  const day = 1000 * 60 * 60 * 24
  const date = Date.now()
  const today = date - day
  const lastWeek = date - day * 7
  const lastMonth = date - day * 30
  const lastYear = date - day * 365

  const list = Object.entries(
    Object.groupBy(feedEntries, (item: FeedEntry) => {
      const publishedAt = Date.parse(item.publishedAt)
      return publishedAt > today
        ? 'Today'
        : publishedAt > lastWeek
          ? 'This Week'
          : publishedAt > lastMonth
            ? 'This Month'
            : publishedAt > lastYear
              ? 'This Year'
              : 'This Lifetime'
    }),
  )

  const { ref } = useInView({
    threshold: 0,
    onChange: (inView) => inView && loadMore && loadMore(),
  })

  return (
    <Box spaceY={6} px={8} pb={8}>
      {list.map(([title, feedEntries]) => (
        <Box key={title} spaceY={6}>
          <HStack spaceX={4}>
            <Divider flex={1} />
            <Text as="span" fontSize="sm" fontWeight="medium">
              {title}
            </Text>
            <Divider flex={1} />
          </HStack>
          <Container spaceY={4}>
            {feedEntries.map((feedEntry, i) => (
              <Box
                key={feedEntry.id}
                ref={hasMore && i === feedEntries.length - 1 ? ref : undefined}
              >
                <FeedEntryCard feedEntry={feedEntry} />
              </Box>
            ))}
          </Container>
        </Box>
      ))}
    </Box>
  )
}
