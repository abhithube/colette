import {
	Header,
	HeaderActionGroup,
	HeaderActionItem,
	HeaderTitle,
} from '@/components/header'
import {
	ensureInfiniteQueryData,
	getFeedOptions,
	listEntriesOptions,
} from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { CircleX, ExternalLink, ListChecks, Tags } from 'lucide-react'
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
				<HeaderActionGroup>
					<HeaderActionItem asChild>
						<a href={feed.link} target="_blank" rel="noreferrer">
							<ExternalLink className="h-4 w-4 shrink-0" />
							<span>Open Link</span>
						</a>
					</HeaderActionItem>
					<HeaderActionItem>
						<Tags className="h-4 w-4 shrink-0" />
						<span>Edit Tags</span>
					</HeaderActionItem>
					<HeaderActionItem>
						<ListChecks className="h-4 w-4 shrink-0" />
						<span>Mark as Read</span>
					</HeaderActionItem>
					<HeaderActionItem variant="destructive">
						<CircleX className="h-4 w-4 shrink-0" />
						<span>Unsubscribe</span>
					</HeaderActionItem>
				</HeaderActionGroup>
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
