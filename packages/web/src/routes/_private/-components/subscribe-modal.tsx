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
import { client } from '@/lib/client'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../feeds'

const formSchema = z.object({
	url: z.string().url(),
})

type Props = {
	close: () => void
}

export function SubscribeModal({ close }: Props) {
	const { profile } = Route.useRouteContext()

	const [loading, setLoading] = useState(false)

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			url: '',
		},
	})

	const navigate = useNavigate()
	const queryClient = useQueryClient()

	const { mutateAsync: createFeed } = useMutation({
		mutationFn: async (values: z.infer<typeof formSchema>) => {
			const res = await client.POST('/api/v1/feeds', {
				body: {
					url: values.url,
				},
			})

			return res.data
		},
		onMutate: () => {
			setLoading(true)
		},
		onSuccess: async (data) => {
			form.reset()
			setLoading(false)
			close()

			await queryClient.invalidateQueries({
				queryKey: ['/profiles', profile.id, '/feeds'],
			})

			if (data) {
				await navigate({
					to: '/feeds/$id',
					params: {
						id: data.id,
					},
				})
			}
		},
	})

	return (
		<DialogContent className="max-w-[400px]">
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
						<Button disabled={loading}>
							{loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
							Submit
						</Button>
					</DialogFooter>
				</form>
			</Form>
		</DialogContent>
	)
}
