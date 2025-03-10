import { CollectionsSection } from './collections/collections-section'
import { StreamsSection } from './streams/streams-section'
import { UserCard } from './user-card'
import type { User } from '@colette/core'
import { Archive, Home, Rss } from 'lucide-react'
import type { FC } from 'react'
import { Link } from 'wouter'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '~/components/ui/sidebar'

export const AppSidebar: FC<{ user: User }> = (props) => {
  return (
    <Sidebar>
      <SidebarHeader>
        <span>Colette</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link to="/">
                    <Home />
                    Home
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link to="/subscriptions">
                    <Rss />
                    Subscriptions
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton asChild>
                  <Link to="/stash">
                    <Archive />
                    Stash
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <StreamsSection />
        <CollectionsSection />
      </SidebarContent>
      <SidebarRail />
      <SidebarFooter>
        <UserCard user={props.user} />
      </SidebarFooter>
    </Sidebar>
  )
}
