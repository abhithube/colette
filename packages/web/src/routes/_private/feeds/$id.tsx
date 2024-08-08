import {
	Header,
	HeaderActionGroup,
	HeaderActionItem,
	HeaderTitle,
} from '@/components/header'
import { Icon } from '@/components/icon'
import {
	ensureInfiniteQueryData,
	getFeedOptions,
	listEntriesOptions,
} from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { CircleX, ExternalLink, ListChecks, Tags } from 'lucide-react'
import { useEffect, useState } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'
import { UnsubscribeAlert } from './-components/unsubscribe-alert'

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

	const [isUnsubscribeAlertOpen, setUnsubscribeAlertOpen] = useState(false)

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
							<Icon value={ExternalLink} />
							<span>Open Link</span>
						</a>
					</HeaderActionItem>
					<HeaderActionItem>
						<Icon value={Tags} />
						<span>Edit Tags</span>
					</HeaderActionItem>
					<HeaderActionItem>
						<Icon value={ListChecks} />
						<span>Mark as Read</span>
					</HeaderActionItem>
					<HeaderActionItem
						variant="destructive"
						onClick={(e) => {
							e.stopPropagation()

							setUnsubscribeAlertOpen(true)
						}}
					>
						<Icon value={CircleX} />
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
			<UnsubscribeAlert
				feed={feed}
				isOpen={isUnsubscribeAlertOpen}
				setOpen={setUnsubscribeAlertOpen}
			/>
		</>
	)
}
