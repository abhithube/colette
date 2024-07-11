import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/feeds')({
	component: () => <div>/feeds</div>,
})
