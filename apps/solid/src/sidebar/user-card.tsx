import type { User as AppUser } from '@colette/core'
import ChevronsUpDown from 'lucide-solid/icons/chevrons-up-down'
import User from 'lucide-solid/icons/user'
import type { Component } from 'solid-js'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar'

export const UserCard: Component<{ user: AppUser }> = ({ user }) => {
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu sameWidth>
          <DropdownMenuTrigger
            as={SidebarMenuButton<'button'>}
            size="lg"
            class="data-[expanded]:bg-sidebar-accent"
          >
            <div class="flex size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
              <User class="size-4" />
            </div>
            <span class="font-semibold">{user.email.split('@')[0]}</span>
            <ChevronsUpDown class="ml-auto" />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>Logout</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
