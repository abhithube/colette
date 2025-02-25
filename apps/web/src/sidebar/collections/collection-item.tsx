import { Collection } from '@colette/core'
import { Library } from 'lucide-react'
import { FC } from 'react'
import { Link } from 'wouter'
import { SidebarMenuButton, SidebarMenuItem } from '~/components/ui/sidebar'

export const CollectionItem: FC<{ collection: Collection }> = (props) => {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton asChild>
        <Link to={`/collections/${props.collection.id}`}>
          <Library />
          {props.collection.title}
        </Link>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}
