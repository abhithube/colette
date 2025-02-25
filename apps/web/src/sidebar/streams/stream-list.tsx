import { StreamItem } from './stream-item'
import { listStreamsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'
import { SidebarMenu } from '~/components/ui/sidebar'

export const StreamList: FC = () => {
  const api = useAPI()

  const { data: streams, isLoading } = useQuery(listStreamsOptions(api))

  if (isLoading || !streams) return

  return (
    <SidebarMenu>
      {streams.data.map((stream) => (
        <StreamItem key={stream.id} stream={stream} />
      ))}
    </SidebarMenu>
  )
}
