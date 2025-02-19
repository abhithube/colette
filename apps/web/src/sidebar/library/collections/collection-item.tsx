import { DeleteCollectionAlert } from './delete-collection-alert'
import type { Collection } from '@colette/core'
import { Library, MoreHorizontal } from 'lucide-react'
import { type FC, useState } from 'react'
import { Link, useRoute } from 'wouter'
import { SidebarMenuSubAction } from '~/components/sidebar-menu-sub-action'
import { AlertDialog, AlertDialogTrigger } from '~/components/ui/alert-dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '~/components/ui/sidebar'

export const CollectionItem: FC<{ collection: Collection }> = (props) => {
  const [match, params] = useRoute('/collections/:id')

  const [isOpen, setOpen] = useState(false)

  return (
    <SidebarMenuSubItem>
      <SidebarMenuSubButton
        asChild
        isActive={match && props.collection.id === params?.id}
      >
        <Link to={`/collections/${props.collection.id}`}>
          <Library className="text-primary" />
          <span className="line-clamp-1">{props.collection.title}</span>
        </Link>
      </SidebarMenuSubButton>
      <AlertDialog open={isOpen} onOpenChange={setOpen}>
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger asChild>
            <SidebarMenuSubAction showOnHover>
              <MoreHorizontal />
            </SidebarMenuSubAction>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="right">
            <AlertDialogTrigger asChild>
              <DropdownMenuItem>Delete</DropdownMenuItem>
            </AlertDialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
        <DeleteCollectionAlert
          collection={props.collection}
          close={() => setOpen(false)}
        />
      </AlertDialog>
    </SidebarMenuSubItem>
  )
}
