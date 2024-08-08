import { Icon } from '@/components/icon'
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
import { Input } from '@/components/ui/input'
import { createBookmarkOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../bookmarks'

const formSchema = z.object({
	url: z.string().url(),
})

type Props = {
	close: () => void
}

export function AddBookmarkModal({ close }: Props) {
	const context = Route.useRouteContext()

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			url: '',
		},
	})

	const navigate = useNavigate()

	const { mutateAsync: createBookmark, isPending } = useMutation(
		createBookmarkOptions(
			{
				onSuccess: () => {
					form.reset()
					close()

					navigate({
						to: '/bookmarks/stash',
					})
				},
			},
			context.api,
		),
	)

	return (
		<DialogContent>
			<Form {...form}>
				<form onSubmit={form.handleSubmit((data) => createBookmark(data))}>
					<DialogHeader>
						<DialogTitle>Add Bookmark</DialogTitle>
						<DialogDescription>Add a bookmark to the stash.</DialogDescription>
					</DialogHeader>
					<div className="flex flex-col space-y-4 py-4">
						<FormField
							control={form.control}
							name="url"
							render={({ field }) => (
								<FormItem>
									<FormLabel>URL</FormLabel>
									<Input {...field} />
									<FormDescription>URL of the bookmark</FormDescription>
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
