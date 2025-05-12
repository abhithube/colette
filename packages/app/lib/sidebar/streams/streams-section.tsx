import { StreamList } from './stream-list'
import { Sidebar } from '@colette/ui'
import { Plus } from 'lucide-react'

export const StreamsSection = () => {
  return (
    <Sidebar.Group>
      <Sidebar.GroupLabel>Streams</Sidebar.GroupLabel>
      <Sidebar.GroupAction>
        <Plus />
      </Sidebar.GroupAction>
      <Sidebar.GroupContent>
        <StreamList />
      </Sidebar.GroupContent>
    </Sidebar.Group>
  )
}
