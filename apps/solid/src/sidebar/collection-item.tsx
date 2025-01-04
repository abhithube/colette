import type { Collection } from '@colette/core'
import { A } from '@solidjs/router'
import Library from 'lucide-solid/icons/library'
import MoreHorizontal from 'lucide-solid/icons/more-horizontal'
import type { Component } from 'solid-js'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar'

export const CollectionItem: Component<{ collection: Collection }> = (
  props,
) => {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton as={A} href={`/collections/${props.collection.id}`}>
        <Library class="text-orange-500" />
        {props.collection.title}
      </SidebarMenuButton>
      <DropdownMenu placement="right-start">
        <DropdownMenuTrigger as={SidebarMenuAction<'button'>}>
          <MoreHorizontal />
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem>Delete</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuItem>
  )
}
