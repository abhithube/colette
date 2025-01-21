import { DeleteCollectionAlert } from './delete-collection-alert'
import type { Collection } from '@colette/core'
import { A } from '@solidjs/router'
import Library from 'lucide-solid/icons/library'
import MoreHorizontal from 'lucide-solid/icons/more-horizontal'
import { type Component, createSignal } from 'solid-js'
import { AlertDialog, AlertDialogTrigger } from '~/components/ui/alert-dialog'
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
  const [isAlertOpen, setAlertOpen] = createSignal(false)

  return (
    <SidebarMenuItem>
      <SidebarMenuButton as={A} href={`/collections/${props.collection.id}`}>
        <Library class="text-orange-500" />
        {props.collection.title}
      </SidebarMenuButton>
      <AlertDialog open={isAlertOpen()} onOpenChange={setAlertOpen}>
        <DropdownMenu placement="right-start">
          <DropdownMenuTrigger as={SidebarMenuAction<'button'>}>
            <MoreHorizontal />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <AlertDialogTrigger as={DropdownMenuItem<'div'>}>
              Delete
            </AlertDialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
        <DeleteCollectionAlert
          collection={props.collection}
          close={() => setAlertOpen(false)}
        />
      </AlertDialog>
    </SidebarMenuItem>
  )
}
