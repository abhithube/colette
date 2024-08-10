import { Header, HeaderTitle } from '@/components/header'
import {
  ensureInfiniteQueryData,
  getTagOptions,
  listEntriesOptions,
} from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/tags/$id')({
  loader: async ({ context, params }) => {
    const tagOptions = getTagOptions(params.id, context.api)
    const tag = await context.queryClient.ensureQueryData(tagOptions)

    const entryOptions = listEntriesOptions(
      {
        hasRead: false,
        'tag[]': [tag.title],
      },
      context.profile.id,
      context.api,
    )
    await ensureInfiniteQueryData(context.queryClient, entryOptions as any)

    return {
      entryOptions,
      tagOptions,
    }
  },
  component: Component,
})

function Component() {
  const { entryOptions, tagOptions } = Route.useLoaderData()

  const {
    data: entries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(entryOptions)
  const { data: tag } = useQuery(tagOptions)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!entries || !tag) return

  return (
    <>
      <Header>
        <HeaderTitle>{tag.title}</HeaderTitle>
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
