import { StreamItem } from './stream-item'
import { listStreamsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'

export const StreamList = () => {
  const api = useAPI()

  const query = useQuery(listStreamsOptions(api))

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.data.map((stream) => (
        <StreamItem key={stream.id} stream={stream} />
      ))}
    </Sidebar.Menu>
  )
}
