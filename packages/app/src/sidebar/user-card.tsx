import { useAPI } from '../lib/api-context'
import type { User as AppUser } from '@colette/core'
import { logoutOptions } from '@colette/query'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@colette/react-ui/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@colette/react-ui/components/ui/sidebar'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ChevronsUpDown, User } from 'lucide-react'
import type { FC } from 'react'

export const UserCard: FC<{ user: AppUser }> = (props) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  const mutation = useMutation(logoutOptions(api, queryClient))

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              className="data-[expanded]:bg-sidebar-accent"
              size="lg"
            >
              <div className="bg-sidebar-primary text-sidebar-primary-foreground flex size-8 items-center justify-center rounded-lg">
                <User className="size-4" />
              </div>
              <span className="font-semibold">
                {props.user.email.split('@')[0]}
              </span>
              <ChevronsUpDown className="ml-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-[--radix-dropdown-menu-trigger-width]">
            <DropdownMenuItem onSelect={() => mutation.mutate()}>
              Logout
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
