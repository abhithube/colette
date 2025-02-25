import { StreamList } from './stream-list'
import { Plus } from 'lucide-react'
import { FC } from 'react'
import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
} from '~/components/ui/sidebar'

export const StreamsSection: FC = () => {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Streams</SidebarGroupLabel>
      <SidebarGroupAction>
        <Plus />
      </SidebarGroupAction>
      <SidebarGroupContent>
        <StreamList />
      </SidebarGroupContent>
    </SidebarGroup>
  )
}
