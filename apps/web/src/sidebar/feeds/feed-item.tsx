import { UnsubscribeAlert } from './unsubscribe-alert'
import type { Feed } from '@colette/core'
import { MoreHorizontal } from 'lucide-react'
import { type FC, useState } from 'react'
import { Link, useRoute } from 'wouter'
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

export const FeedItem: FC<{ feed: Feed }> = (props) => {
  const [match, params] = useRoute('/feeds/:id')

  const [isOpen, setOpen] = useState(false)

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        asChild
        isActive={match && props.feed.id === params?.id}
      >
        <Link className="justify-between" to={`/feeds/${props.feed.id}`}>
          <Favicon url={props.feed.link} />
          <span className="line-clamp-1">{props.feed.title}</span>
          {props.feed.unreadCount && (
            <Badge className="justify-self-end" variant="outline">
              {props.feed.unreadCount}
            </Badge>
          )}
        </Link>
      </SidebarMenuButton>
      <AlertDialog open={isOpen} onOpenChange={setOpen}>
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger asChild>
            <SidebarMenuAction>
              <MoreHorizontal />
            </SidebarMenuAction>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="right">
            <AlertDialogTrigger asChild>
              <DropdownMenuItem>Unsubscribe</DropdownMenuItem>
            </AlertDialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
        <UnsubscribeAlert feed={props.feed} close={() => setOpen(false)} />
      </AlertDialog>
    </SidebarMenuItem>
  )
}
