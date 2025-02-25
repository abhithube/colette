import { StreamItem } from './stream-item'
import { listStreamsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'
import { SidebarMenu } from '~/components/ui/sidebar'

export const StreamList: FC = () => {
  const api = useAPI()

  const query = useQuery(listStreamsOptions(api))

  if (query.isLoading || !query.data) return

  return (
    <SidebarMenu>
      {query.data.data.map((stream) => (
        <StreamItem key={stream.id} stream={stream} />
      ))}
    </SidebarMenu>
  )
}
