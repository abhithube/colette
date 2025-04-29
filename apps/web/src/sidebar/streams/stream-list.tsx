import { StreamItem } from './stream-item'
import { listStreamsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useQuery } from '@tanstack/react-query'
import { getRouteApi } from '@tanstack/react-router'

const routeApi = getRouteApi('/layout')

export const StreamList = () => {
  const context = routeApi.useRouteContext()

  const query = useQuery(listStreamsOptions(context.api))

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.data.map((stream) => (
        <StreamItem key={stream.id} stream={stream} />
      ))}
    </Sidebar.Menu>
  )
}
