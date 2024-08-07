import {
	ensureInfiniteQueryData,
	getTagOptions,
	listBookmarksOptions,
} from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { BookmarkGrid } from './-components/bookmark-grid'

export const Route = createFileRoute('/_private/bookmarks/tags/$id')({
	loader: async ({ context, params }) => {
		const bookmarkOptions = listBookmarksOptions(
			{ 'tag[]': [params.id] },
			context.profile.id,
			context.api,
		)
		const tagOptions = getTagOptions(params.id, context.api)

		await Promise.all([
			ensureInfiniteQueryData(context.queryClient, bookmarkOptions as any),
			context.queryClient.ensureQueryData(tagOptions),
		])

		return {
			bookmarkOptions,
			tagOptions,
		}
	},
	component: Component,
})

function Component() {
	const { bookmarkOptions, tagOptions } = Route.useLoaderData()

	const {
		data: bookmarks,
		hasNextPage,
		fetchNextPage,
	} = useInfiniteQuery(bookmarkOptions)
	const { data: tag } = useQuery(tagOptions)

	useEffect(() => {
		window.scrollTo(0, 0)
	}, [])

	if (!bookmarks || !tag) return

	return (
		<>
			<header className="sticky top-0 w-full bg-background p-8">
				<h1 className="truncate font-medium text-3xl">{tag.title}</h1>
			</header>
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
