import { ensureInfiniteQueryData, listEntriesOptions } from '@/lib/query'
import { queryOptions, useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { EntryGrid } from '../-components/entry-grid'

export const Route = createFileRoute('/_private/feeds/$id')({
	loader: async ({ context, params }) => {
		const feedOptions = queryOptions({
			queryKey: ['feeds', params.id],
			queryFn: ({ signal }) =>
				context.api.feeds.get(params.id, {
					signal,
				}),
		})

		await Promise.all([
			context.queryClient.ensureQueryData(feedOptions),
			ensureInfiniteQueryData(
				context.queryClient,
				listEntriesOptions({
					feedId: params.id,
				}) as any,
			),
		])

		return {
			feedOptions,
		}
	},
	component: Component,
})

function Component() {
	const { id } = Route.useParams()
	const { feedOptions } = Route.useLoaderData()

	const { data: feed } = useQuery(feedOptions)
	const {
		data: entries,
		hasNextPage,
		fetchNextPage,
	} = useInfiniteQuery(
		listEntriesOptions({
			feedId: id,
		}),
	)

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		window.scrollTo(0, 0)
	}, [id])

	if (!feed || !entries) return

	return (
		<>
			<header className="sticky top-0 w-full bg-background p-8">
				<h1 className="truncate font-medium text-3xl">{feed.title}</h1>
			</header>
			<main className="pb-8">
				<EntryGrid
					entries={entries.pages.flatMap((page) => page.data)}
					hasMore={hasNextPage}
					loadMore={fetchNextPage}
				/>
			</main>
		</>
	)
}
