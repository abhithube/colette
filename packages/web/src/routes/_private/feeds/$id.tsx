import { client } from '@/lib/client'
import { ensureInfiniteQueryData, listEntriesOptions } from '@/lib/query'
import { queryOptions, useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { EntryGrid } from '../-components/entry-grid'

const feedOptions = (id: string) =>
	queryOptions({
		queryKey: ['/feeds', id],
		queryFn: async ({ signal }) => {
			const res = await client.GET('/api/v1/feeds/{id}', {
				params: {
					path: {
						id,
					},
				},
				signal,
			})

			return res.data
		},
	})

export const Route = createFileRoute('/_private/feeds/$id')({
	loader: async ({ context, params }) => {
		await Promise.all([
			context.queryClient.ensureQueryData(feedOptions(params.id)),
			ensureInfiniteQueryData(
				context.queryClient,
				listEntriesOptions({
					feedId: params.id,
				}) as any,
			),
		])
	},
	component: Component,
})

function Component() {
	const { id } = Route.useParams()
	const { data: feed } = useQuery(feedOptions(id))
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
