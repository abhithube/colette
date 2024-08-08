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
import { BadGatewayError, UnprocessableContentError } from '@colette/openapi'
import { createFeedOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../feeds'

const formSchema = z.object({
	url: z.string().url(),
})

type Props = {
	close: () => void
}

export function SubscribeModal({ close }: Props) {
	const context = Route.useRouteContext()

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			url: '',
		},
	})

	const navigate = useNavigate()

	const { mutateAsync: createFeed, isPending } = useMutation(
		createFeedOptions(
			{
				onSuccess: async (data) => {
					form.reset()
					close()

					await context.queryClient.invalidateQueries({
						queryKey: ['profiles', context.profile.id, 'feeds'],
					})

					await navigate({
						to: '/feeds/$id',
						params: {
							id: data.id,
						},
					})
				},
				onError: (error) => {
					if (error instanceof UnprocessableContentError) {
						form.setError('root', {
							message: error.message,
						})
					} else if (error instanceof BadGatewayError) {
						form.setError('root', {
							message: error.message,
						})
					}
				},
			},
			context.api,
		),
	)

	return (
		<DialogContent>
			<Form {...form}>
				<form onSubmit={form.handleSubmit((data) => createFeed(data))}>
					<DialogHeader>
						<DialogTitle>Add Feed</DialogTitle>
						<DialogDescription>
							Subscribe to a RSS or Atom feed and receive the latest updates.
						</DialogDescription>
					</DialogHeader>
					<div className="flex flex-col py-4">
						<FormField
							control={form.control}
							name="url"
							render={({ field }) => (
								<FormItem>
									<FormLabel>URL</FormLabel>
									<Input {...field} />
									<FormDescription>URL of the RSS or Atom Feed</FormDescription>
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
