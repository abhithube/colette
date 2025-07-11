import { StreamItem } from './stream-item'
import { listStreamsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useQuery } from '@tanstack/react-query'

export const StreamList = () => {
  const query = useQuery(listStreamsOptions())

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.items.map((stream) => (
        <StreamItem key={stream.id} stream={stream as never} />
      ))}
    </Sidebar.Menu>
  )
}
