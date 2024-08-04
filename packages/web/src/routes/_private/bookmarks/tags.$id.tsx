import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/bookmarks/tags/$id')({
	component: () => Component,
})

function Component() {
	const { id } = Route.useParams()

	return <div>/bookmarks/tags/{id}</div>
}
