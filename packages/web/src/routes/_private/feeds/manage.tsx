import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_private/feeds/manage')({
  component: () => <div>/feeds/manage</div>,
})
