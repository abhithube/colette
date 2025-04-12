import { CollectionList } from './collection-list'
import { Sidebar } from '@colette/ui'
import { Plus } from 'lucide-react'

export const CollectionsSection = () => {
  return (
    <Sidebar.Group>
      <Sidebar.GroupLabel>Collections</Sidebar.GroupLabel>
      <Sidebar.GroupAction>
        <Plus />
      </Sidebar.GroupAction>
      <Sidebar.GroupContent>
        <CollectionList />
      </Sidebar.GroupContent>
    </Sidebar.Group>
  )
}
