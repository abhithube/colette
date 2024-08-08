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
		const entryOptions = listEntriesOptions(
			{
				hasRead: false,
				'tag[]': [params.id],
			},
			context.profile.id,
			context.api,
		)
		const tagOptions = getTagOptions(params.id, context.api)

		await Promise.all([
			ensureInfiniteQueryData(context.queryClient, entryOptions as any),
			context.queryClient.ensureQueryData(tagOptions),
		])

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
			<main className="pb-8">
				<FeedEntryGrid
					entries={entries.pages.flatMap((page) => page.data)}
					hasMore={hasNextPage}
					loadMore={fetchNextPage}
				/>
			</main>
		</>
	)
}
