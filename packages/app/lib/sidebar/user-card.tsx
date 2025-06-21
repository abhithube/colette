import client from '@colette/core/client'
import { useLogoutUserMutation } from '@colette/query'
import { useRouter } from '@colette/router'
import { Menu, Sidebar } from '@colette/ui'
import { useUser } from '@colette/util'
import { ChevronsUpDown, User } from 'lucide-react'

export const UserCard = () => {
  const router = useRouter()
  const user = useUser()

  const logoutUser = useLogoutUserMutation()

  function onLogout() {
    logoutUser.mutate(undefined, {
      onSuccess: () => {
        client.setConfig({
          ...client.getConfig(),
          accessToken: undefined,
        })

        router.navigate({
          to: '/login',
          state: {
            loggedOut: true,
          },
          replace: true,
        })
      },
    })
  }

  return (
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Menu.Root positioning={{ sameWidth: true }}>
          <Menu.Trigger asChild>
            <Sidebar.MenuButton
              className="data-[expanded]:bg-sidebar-accent"
              size="lg"
            >
              <div className="bg-sidebar-primary text-sidebar-primary-foreground flex size-8 items-center justify-center rounded-lg">
                <User className="size-4" />
              </div>
              <span className="font-semibold">
                {user.displayName ?? user.email?.split('@')[0]}
              </span>
              <ChevronsUpDown className="ml-auto" />
            </Sidebar.MenuButton>
          </Menu.Trigger>
          <Menu.Content>
            <Menu.Item value="logout" onSelect={onLogout}>
              Logout
            </Menu.Item>
          </Menu.Content>
        </Menu.Root>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  )
}
