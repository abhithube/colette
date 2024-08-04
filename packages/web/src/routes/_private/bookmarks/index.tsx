import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/bookmarks/')({
	component: () => <div>/bookmarks</div>,
})
