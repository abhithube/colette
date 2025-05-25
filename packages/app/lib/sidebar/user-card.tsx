import { Menu, Sidebar } from '@colette/ui'
import { useOIDCConfig, useUser } from '@colette/util'
import { ChevronsUpDown, User } from 'lucide-react'
import * as client from 'openid-client'

export const UserCard = () => {
  const oidcConfig = useOIDCConfig()
  const user = useUser()

  async function onLogout() {
    const refreshToken = localStorage.getItem('colette-refresh-token')
    if (refreshToken) {
      await client.tokenRevocation(oidcConfig.clientConfig, refreshToken, {
        token_type_hint: 'refresh_token',
      })
    }

    localStorage.removeItem('colette-access-token')
    localStorage.removeItem('colette-refresh-token')

    window.location.href = '/login?loggedOut=true'
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
                {user.displayName ??
                  user.email?.split('@')[0] ??
                  user.externalId}
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
