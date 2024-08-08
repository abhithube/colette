import { Icon } from '@/components/icon'
import { MultiSelect } from '@/components/multi-select'
import { Button } from '@/components/ui/button'
import {
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import {
	Form,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
} from '@/components/ui/form'
import type { Feed } from '@colette/openapi'
import { listTagsOptions, updateFeedOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../../_private'

const formSchema = z.object({
	tags: z.string().array(),
})

type Props = {
	feed: Feed
	close: () => void
}

export function EditFeedModal({ feed, close }: Props) {
	const context = Route.useRouteContext()

	const { data: tags } = useQuery(
		listTagsOptions({}, context.profile.id, context.api),
	)

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
	})

	const { mutateAsync: updateFeed, isPending } = useMutation(
		updateFeedOptions(
			{
				onSuccess: async (data) => {
					await context.queryClient.setQueryData(['feeds', feed.id], data)

					close()
				},
			},
			context.api,
		),
	)

	useEffect(() => {
		form.reset({
			tags: feed.tags?.map((tag) => tag.title) ?? [],
		})
	}, [form, feed])

	if (!tags) return

	return (
		<DialogContent>
			<Form {...form}>
				<form
					onSubmit={form.handleSubmit((data) =>
						updateFeed({
							id: feed.id,
							body: {
								tags: data.tags.map((title) => ({ title })),
							},
						}),
					)}
				>
					<DialogHeader>
						<DialogTitle>Edit {feed.title}</DialogTitle>
						<DialogDescription>Edit a feed's data.</DialogDescription>
					</DialogHeader>
					<div className="flex flex-col space-y-4 py-4">
						<FormField
							control={form.control}
							name="tags"
							render={({ field }) => (
								<FormItem className="flex flex-col">
									<FormLabel>Tags</FormLabel>
									<MultiSelect
										options={tags.data.map(({ title }) => ({
											value: title,
											label: title,
										}))}
										value={field.value}
										onChange={(value) => {
											form.setValue('tags', value)
										}}
									/>
									<FormDescription>Tags to add to the feed</FormDescription>
								</FormItem>
							)}
						/>
					</div>
					<DialogFooter>
						<Button disabled={isPending}>
							{isPending && (
								<Icon className="mr-2 animate-spin" value={Loader2} />
							)}
							Submit
						</Button>
					</DialogFooter>
				</form>
			</Form>
		</DialogContent>
	)
}
