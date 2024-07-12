import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/feeds/$id')({
	component: Component,
})

function Component() {
	const { id } = Route.useParams()
	return <div>/feeds/{id}</div>
}
