import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { client } from '@/lib/client'
import type { Feed } from '@/lib/types'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useMatchRoute, useNavigate } from '@tanstack/react-router'
import { object } from 'zod'

export function UnsubscribeAlert({
	feed,
	isOpen,
	setOpen,
}: {
	feed: Feed
	isOpen: boolean
	setOpen: React.Dispatch<React.SetStateAction<boolean>>
}) {
	const navigate = useNavigate()

	const matchRoute = useMatchRoute()
	const params = matchRoute({ to: '/feeds/$id' })

	const queryClient = useQueryClient()

	const { mutateAsync: unsubscribe } = useMutation({
		mutationFn: async () => {
			const res = await client.DELETE('/api/v1/feeds/{id}', {
				params: {
					path: {
						id: feed.id,
					},
				},
			})
			if (res.error || !res.data) throw new Error()
		},
		onSuccess: async () => {
			if (params instanceof object && params.id === feed.id) {
				await navigate({
					to: '/feeds',
				})
			}

			await queryClient.invalidateQueries({
				queryKey: ['/feeds'],
			})
		},
	})

	return (
		<AlertDialog open={isOpen} onOpenChange={setOpen}>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>
						Unsubscribe from {feed.customTitle ?? feed.title}?
					</AlertDialogTitle>
					<AlertDialogDescription>
						Are you sure you want to unsubscribe? This action cannot be undone.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel onClick={(e) => e.stopPropagation()}>
						Cancel
					</AlertDialogCancel>
					<AlertDialogAction
						onClick={(e) => {
							e.stopPropagation()

							unsubscribe()
						}}
					>
						Continue
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
