import { ensureInfiniteQueryData, listEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { EntryGrid } from '../-components/entry-grid'

export const Route = createFileRoute('/_private/feeds/')({
	loader: async ({ context }) => {
		const options = listEntriesOptions({}, context.profile.id, context.api)

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
		data: entries,
		hasNextPage,
		fetchNextPage,
	} = useInfiniteQuery(options)

	useEffect(() => {
		window.scrollTo(0, 0)
	}, [])

	if (!entries) return

	return (
		<>
			<header className="sticky top-0 w-full bg-background p-8">
				<h1 className="font-medium text-3xl">All Feeds</h1>
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
