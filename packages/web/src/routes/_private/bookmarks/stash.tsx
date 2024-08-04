import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/bookmarks/stash')({
	component: () => <div>/bookmarks/stash</div>,
})
