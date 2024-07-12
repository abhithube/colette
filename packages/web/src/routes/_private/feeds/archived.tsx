import { ensureInfiniteQueryData, listEntriesOptions } from '@/lib/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { EntryGrid } from '../-components/entry-grid'

export const Route = createFileRoute('/_private/feeds/archived')({
	loader: async ({ context }) => {
		await ensureInfiniteQueryData(
			context.queryClient,
			listEntriesOptions({
				hasRead: true,
			}) as any,
		)
	},
	component: Component,
})

function Component() {
	const { data, hasNextPage, fetchNextPage } = useInfiniteQuery(
		listEntriesOptions({
			hasRead: true,
		}),
	)

	useEffect(() => {
		window.scrollTo(0, 0)
	}, [])

	if (!data) return

	const entries = data.pages.flatMap((page) => page.data)

	return (
		<>
			<header className="sticky top-0 w-full bg-background p-8">
				<h1 className="font-medium text-3xl">Archived</h1>
			</header>
			<main className="pb-8">
				{entries.length === 0 && (
					<div className="mx-8">
						<span className="text-muted-foreground">No archived entries.</span>
					</div>
				)}
				<EntryGrid
					entries={entries}
					hasMore={hasNextPage}
					loadMore={fetchNextPage}
				/>
			</main>
		</>
	)
}
