import { Header, HeaderTitle } from '@/components/header'
import { ensureInfiniteQueryData, listEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/')({
  loader: async ({ context }) => {
    const options = listEntriesOptions(
      { hasRead: false },
      context.profile.id,
      context.api,
    )

    await ensureInfiniteQueryData(context.queryClient, options as any)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const {
    data: entries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(options)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!entries) return

  return (
    <>
      <Header>
        <HeaderTitle>All Feeds</HeaderTitle>
      </Header>
      <main>
        <FeedEntryGrid
          entries={entries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
