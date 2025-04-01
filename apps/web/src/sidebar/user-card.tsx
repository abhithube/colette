import type { User as AppUser } from '@colette/core'
import { useLogoutUserMutation } from '@colette/query'
import { ChevronsUpDown, User } from 'lucide-react'
import type { FC } from 'react'
import { navigate } from 'wouter/use-browser-location'
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

export const UserCard: FC<{ user: AppUser }> = (props) => {
  const logoutUser = useLogoutUserMutation()

  function onLogout() {
    logoutUser.mutate(undefined, {
      onSuccess: () =>
        navigate('/login', {
          replace: true,
        }),
    })
  }

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
          <DropdownMenuContent className="w-[var(--radix-dropdown-menu-trigger-width)]">
            <DropdownMenuItem onSelect={onLogout}>Logout</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
