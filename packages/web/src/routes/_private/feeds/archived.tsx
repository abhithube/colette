import { Header, HeaderTitle } from '@/components/header'
import { ensureInfiniteQueryData, listEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/archived')({
  loader: async ({ context }) => {
    const options = listEntriesOptions(
      {
        hasRead: true,
      },
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

  const { data, hasNextPage, fetchNextPage } = useInfiniteQuery(options)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!data) return

  const entries = data.pages.flatMap((page) => page.data)

  return (
    <>
      <Header>
        <HeaderTitle>Archived</HeaderTitle>
      </Header>
      <main>
        {entries.length === 0 && (
          <div className="mx-8">
            <span className="text-muted-foreground">No archived entries.</span>
          </div>
        )}
        <FeedEntryGrid
          entries={entries}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
