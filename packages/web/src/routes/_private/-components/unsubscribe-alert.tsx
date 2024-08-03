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
import type { Feed } from '@colette/openapi'
import { deleteFeedOptions } from '@colette/query'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useMatchRoute, useNavigate } from '@tanstack/react-router'
import { Route } from '../feeds'

export function UnsubscribeAlert({
	feed,
	isOpen,
	setOpen,
}: {
	feed: Feed
	isOpen: boolean
	setOpen: React.Dispatch<React.SetStateAction<boolean>>
}) {
	const context = Route.useRouteContext()

	const navigate = useNavigate()

	const matchRoute = useMatchRoute()
	const params = matchRoute({ to: '/feeds/$id' })

	const queryClient = useQueryClient()

	const { mutateAsync: unsubscribe } = useMutation(
		deleteFeedOptions(
			feed.id,
			{
				onSuccess: async () => {
					if (typeof params === 'object' && params.id === feed.id) {
						await navigate({
							to: '/feeds',
						})
					}

					await queryClient.invalidateQueries({
						queryKey: ['profiles', context.profile.id, 'feeds'],
					})
				},
			},
			context.api,
		),
	)

	return (
		<AlertDialog open={isOpen} onOpenChange={setOpen}>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Unsubscribe from {feed.title}?</AlertDialogTitle>
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
