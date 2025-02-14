import type { User as AppUser } from '@colette/core'
import { logoutOptions } from '@colette/query'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
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
import { useAPI } from '~/lib/api-context'

export const UserCard: Component<{ user: AppUser }> = ({ user }) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  const mutation = createMutation(() => logoutOptions(api, queryClient))

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu sameWidth>
          <DropdownMenuTrigger
            as={SidebarMenuButton<'button'>}
            size="lg"
            class="data-[expanded]:bg-sidebar-accent"
          >
            <div class="bg-sidebar-primary text-sidebar-primary-foreground flex size-8 items-center justify-center rounded-lg">
              <User class="size-4" />
            </div>
            <span class="font-semibold">{user.email.split('@')[0]}</span>
            <ChevronsUpDown class="ml-auto" />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onSelect={mutation.mutate}>
              Logout
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
