import type { Feed } from '@colette/core'
import { A } from '@solidjs/router'
import MoreHorizontal from 'lucide-solid/icons/more-horizontal'
import { type Component, createSignal } from 'solid-js'
import { Favicon } from '~/components/favicon'
import { AlertDialog, AlertDialogTrigger } from '~/components/ui/alert-dialog'
import { Badge } from '~/components/ui/badge'
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
import { UnsubscribeAlert } from './unsubscribe-alert'

export const FeedItem: Component<{ feed: Feed }> = (props) => {
  const [isAlertOpen, setAlertOpen] = createSignal(false)

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        as={A}
        class="justify-between"
        href={`/feeds/${props.feed.id}`}
      >
        <div class="flex items-center gap-2">
          <Favicon url={props.feed.link} />
          {props.feed.title ?? props.feed.originalTitle}
        </div>
        {props.feed.unreadCount && (
          <Badge class="justify-self-end" variant="outline">
            {props.feed.unreadCount}
          </Badge>
        )}
      </SidebarMenuButton>
      <AlertDialog open={isAlertOpen()} onOpenChange={setAlertOpen}>
        <DropdownMenu placement="right-start">
          <DropdownMenuTrigger as={SidebarMenuAction<'button'>}>
            <MoreHorizontal />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <AlertDialogTrigger as={DropdownMenuItem<'div'>}>
              Unsubscribe
            </AlertDialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
        <UnsubscribeAlert feed={props.feed} close={() => setAlertOpen(false)} />
      </AlertDialog>
    </SidebarMenuItem>
  )
}
