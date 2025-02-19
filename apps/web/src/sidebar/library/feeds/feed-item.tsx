import { UnsubscribeAlert } from './unsubscribe-alert'
import type { Feed } from '@colette/core'
import { MoreHorizontal } from 'lucide-react'
import { type FC, useState } from 'react'
import { Link, useRoute } from 'wouter'
import { Favicon } from '~/components/favicon'
import { SidebarMenuSubAction } from '~/components/sidebar-menu-sub-action'
import { AlertDialog, AlertDialogTrigger } from '~/components/ui/alert-dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  SidebarMenuBadge,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '~/components/ui/sidebar'

export const FeedItem: FC<{ feed: Feed }> = (props) => {
  const [match, params] = useRoute('/feeds/:id')

  const [isOpen, setOpen] = useState(false)

  return (
    <SidebarMenuSubItem>
      <SidebarMenuSubButton
        asChild
        isActive={match && props.feed.id === params?.id}
      >
        <Link to={`/feeds/${props.feed.id}`}>
          <Favicon url={props.feed.link} />
          <span className="line-clamp-1">{props.feed.title}</span>
          {props.feed.unreadCount && (
            <SidebarMenuBadge className="group-focus-within/menu-sub-item:opacity-0 group-hover/menu-sub-item:opacity-0 group-has-[button[data-state=open]]/menu-sub-item:opacity-0 md:opacity-100">
              {props.feed.unreadCount}
            </SidebarMenuBadge>
          )}
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
              <DropdownMenuItem>Unsubscribe</DropdownMenuItem>
            </AlertDialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
        <UnsubscribeAlert feed={props.feed} close={() => setOpen(false)} />
      </AlertDialog>
    </SidebarMenuSubItem>
  )
}
