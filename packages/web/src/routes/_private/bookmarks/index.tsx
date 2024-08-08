import { Header, HeaderTitle } from '@/components/header'
import { ensureInfiniteQueryData, listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { BookmarkGrid } from './-components/bookmark-grid'

export const Route = createFileRoute('/_private/bookmarks/')({
	loader: async ({ context }) => {
		const options = listBookmarksOptions({}, context.profile.id, context.api)

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
		data: bookmarks,
		hasNextPage,
		fetchNextPage,
	} = useInfiniteQuery(options)

	useEffect(() => {
		window.scrollTo(0, 0)
	}, [])

	if (!bookmarks) return

	return (
		<>
			<Header>
				<HeaderTitle>All Bookmarks</HeaderTitle>
			</Header>
			<main className="pb-8">
				<BookmarkGrid
					bookmarks={bookmarks.pages.flatMap((page) => page.data)}
					hasMore={hasNextPage}
					loadMore={fetchNextPage}
				/>
			</main>
		</>
	)
}
