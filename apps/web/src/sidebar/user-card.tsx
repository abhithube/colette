import { useLogoutUserMutation } from '@colette/query'
import { Menu, Sidebar } from '@colette/ui'
import { getRouteApi } from '@tanstack/react-router'
import { ChevronsUpDown, User } from 'lucide-react'

const routeApi = getRouteApi('/layout')

export const UserCard = () => {
  const context = routeApi.useRouteContext()
  const navigate = routeApi.useNavigate()

  const logoutUser = useLogoutUserMutation()

  function onLogout() {
    logoutUser.mutate(undefined, {
      onSuccess: () => {
        navigate({
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
                {context.user.email.split('@')[0]}
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
