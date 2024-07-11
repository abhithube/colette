import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/collections')({
	component: () => <div>/collections</div>,
})
