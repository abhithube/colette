import { Stream } from '@colette/core'
import { Podcast } from 'lucide-react'
import { FC } from 'react'
import { Link } from 'wouter'
import { SidebarMenuButton, SidebarMenuItem } from '~/components/ui/sidebar'

export const StreamItem: FC<{ stream: Stream }> = (props) => {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton asChild>
        <Link to={`/streams/${props.stream.id}`}>
          <Podcast />
          {props.stream.title}
        </Link>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}
