import { Header, HeaderTitle } from '@/components/header'
import {
	ensureInfiniteQueryData,
	getFeedOptions,
	listEntriesOptions,
} from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/$id')({
	loader: async ({ context, params }) => {
		const feedOptions = getFeedOptions(params.id, context.api)

		const entryOptions = listEntriesOptions(
			{
				feedId: params.id,
				hasRead: false,
			},
			context.profile.id,
			context.api,
		)

		await Promise.all([
			context.queryClient.ensureQueryData(feedOptions),
			ensureInfiniteQueryData(context.queryClient, entryOptions as any),
		])

		return {
			feedOptions,
			entryOptions,
		}
	},
	component: Component,
})

function Component() {
	const { id } = Route.useParams()
	const { feedOptions, entryOptions } = Route.useLoaderData()

	const { data: feed } = useQuery(feedOptions)
	const {
		data: entries,
		hasNextPage,
		fetchNextPage,
	} = useInfiniteQuery(entryOptions)

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		window.scrollTo(0, 0)
	}, [id])

	if (!feed || !entries) return

	return (
		<>
			<Header>
				<HeaderTitle>{feed.title}</HeaderTitle>
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
