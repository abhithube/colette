import { UnsubscribeAlert } from './unsubscribe-alert'
import type { Feed } from '@colette/core'
import { Favicon } from '@colette/react-ui/components/favicon'
import {
  AlertDialog,
  AlertDialogTrigger,
} from '@colette/react-ui/components/ui/alert-dialog'
import { Badge } from '@colette/react-ui/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@colette/react-ui/components/ui/dropdown-menu'
import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@colette/react-ui/components/ui/sidebar'
import { MoreHorizontal } from 'lucide-react'
import { type FC, useState } from 'react'
import { Link, useRoute } from 'wouter'

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
          <div className="flex items-center gap-2">
            <Favicon url={props.feed.link} />
            {props.feed.title ?? props.feed.originalTitle}
          </div>
          {props.feed.unreadCount && (
            <Badge className="justify-self-end" variant="outline">
              {props.feed.unreadCount}
            </Badge>
          )}
        </Link>
      </SidebarMenuButton>
      <AlertDialog open={isOpen} onOpenChange={setOpen}>
        <DropdownMenu>
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
