import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/feeds/archived')({
	component: () => <div>/feeds/archived</div>,
})
