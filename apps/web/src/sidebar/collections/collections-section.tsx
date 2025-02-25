import { CollectionList } from './collection-list'
import { Plus } from 'lucide-react'
import { FC } from 'react'
import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
} from '~/components/ui/sidebar'

export const CollectionsSection: FC = () => {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Collections</SidebarGroupLabel>
      <SidebarGroupAction>
        <Plus />
      </SidebarGroupAction>
      <SidebarGroupContent>
        <CollectionList />
      </SidebarGroupContent>
    </SidebarGroup>
  )
}
