import { Header, HeaderTitle } from '@/components/header'
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
			<Header>
				<HeaderTitle>{tag.title}</HeaderTitle>
			</Header>
			<main>
				<BookmarkGrid
					bookmarks={bookmarks.pages.flatMap((page) => page.data)}
					hasMore={hasNextPage}
					loadMore={fetchNextPage}
				/>
			</main>
		</>
	)
}
