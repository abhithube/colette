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
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { zfd } from 'zod-form-data'
import { Route } from '../_private'

const formSchema = z.object({
	file: zfd.file(),
})

type Props = {
	close: () => void
}

export function SettingsModal({ close }: Props) {
	const context = Route.useRouteContext()

	const [isLoading, setLoading] = useState(false)

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
	})

	const queryClient = useQueryClient()

	const { mutateAsync: importBackup } = useMutation({
		mutationFn: async (values: z.infer<typeof formSchema>) => {
			await context.api.feeds.import({
				data: await values.file.text(),
			})
		},
		onMutate: () => {
			setLoading(true)
		},
		onSuccess: async () => {
			setLoading(false)
			close()

			await queryClient.invalidateQueries({
				queryKey: ['profiles', context.profile.id, 'feeds'],
			})
		},
	})

	return (
		<DialogContent className="max-w-[425px]">
			<Form {...form}>
				<form onSubmit={form.handleSubmit((data) => importBackup(data))}>
					<DialogHeader>
						<DialogTitle>Import Feeds</DialogTitle>
						<DialogDescription>
							Upload an OPML file to import feeds.
						</DialogDescription>
					</DialogHeader>
					<div className="flex flex-col py-4">
						<FormField
							control={form.control}
							name="file"
							render={({ field }) => (
								<FormItem>
									<FormLabel>File</FormLabel>
									<Input
										type="file"
										onChange={(ev) =>
											field.onChange(
												ev.target.files ? ev.target.files[0] : null,
											)
										}
									/>
									<FormDescription>OPML file to upload</FormDescription>
								</FormItem>
							)}
						/>
					</div>
					<DialogFooter>
						<Button disabled={isLoading}>
							{isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
							Submit
						</Button>
					</DialogFooter>
				</form>
			</Form>
		</DialogContent>
	)
}
