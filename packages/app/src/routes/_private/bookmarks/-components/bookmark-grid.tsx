import type { Bookmark } from '@colette/core'
import { Box, Grid, GridItem } from '@colette/ui'
import { useInView } from 'react-intersection-observer'
import { BookmarkCard } from './bookmark-card'

type Props = {
  bookmarks: Bookmark[]
  hasMore: boolean
  loadMore?: () => void
  created?: Bookmark
}

export function BookmarkGrid({
  bookmarks,
  hasMore = false,
  loadMore,
  created,
}: Props) {
  const { ref } = useInView({
    threshold: 0,
    onChange: (inView) => inView && loadMore && loadMore(),
  })

  const filtered = created
    ? bookmarks.filter((v) => v.id !== created.id)
    : bookmarks

  return (
    <Grid columns={{ base: 1, md: 2, lg: 3 }} gap={4} px={8} pb={8}>
      {created && (
        <Box rounded="lg" border="2">
          <BookmarkCard bookmark={created} />
        </Box>
      )}
      {filtered.map((bookmark, i) => (
        <GridItem
          key={bookmark.id}
          ref={hasMore && i === filtered.length - 1 ? ref : undefined}
        >
          <BookmarkCard bookmark={bookmark} />
        </GridItem>
      ))}
    </Grid>
  )
}
