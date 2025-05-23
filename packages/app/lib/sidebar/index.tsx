import { CollectionsSection } from './collections/collections-section'
import { StreamsSection } from './streams/streams-section'
import { UserCard } from './user-card'
import { Link } from '@colette/router'
import { Sidebar } from '@colette/ui'
import { Archive, Home, Rss } from 'lucide-react'

export const AppSidebar = () => {
  return (
    <Sidebar.Root>
      <Sidebar.Header>
        <span>Colette</span>
      </Sidebar.Header>
      <Sidebar.Content>
        <Sidebar.Group>
          <Sidebar.GroupContent>
            <Sidebar.Menu>
              <Sidebar.MenuItem>
                <Sidebar.MenuButton asChild>
                  <Link to="/">
                    <Home />
                    Home
                  </Link>
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
              <Sidebar.MenuItem>
                <Sidebar.MenuButton asChild>
                  <Link to="/subscriptions">
                    <Rss />
                    Subscriptions
                  </Link>
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
              <Sidebar.MenuItem>
                <Sidebar.MenuButton asChild>
                  <Link to="/stash">
                    <Archive />
                    Stash
                  </Link>
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
            </Sidebar.Menu>
          </Sidebar.GroupContent>
        </Sidebar.Group>
        <StreamsSection />
        <CollectionsSection />
      </Sidebar.Content>
      <Sidebar.Rail />
      <Sidebar.Footer>
        <UserCard />
      </Sidebar.Footer>
    </Sidebar.Root>
  )
}
