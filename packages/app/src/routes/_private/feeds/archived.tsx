import { listFeedEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/archived')({
  loader: ({ context }) => {
    const options = listFeedEntriesOptions(
      {
        hasRead: true,
      },
      context.api,
    )

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

  const feedEntries = data.pages.flatMap((page) => page.data)

  return (
    <>
      <div className="sticky top-0 z-10 flex justify-between bg-background p-8">
        <h1 className="font-medium text-3xl">Archived</h1>
      </div>
      <main>
        <FeedEntryGrid
          feedEntries={feedEntries}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
