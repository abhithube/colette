import { CollectionList } from './collections/collection-list'
import { CreateCollectionModal } from './collections/create-collection-modal'
import { CreateFeedModal } from './feeds/create-form/create-feed-modal'
import { FeedList } from './feeds/feed-list'
import { UserCard } from './user-card'
import type { User } from '@colette/core'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '@colette/react-ui/components/ui/sidebar'
import { Archive, History, Home } from 'lucide-react'
import type { FC } from 'react'
import { Link } from 'wouter'

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
                  <Link to="/archived">
                    <History />
                    Archived
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
        <SidebarGroup>
          <SidebarGroupLabel>Feeds</SidebarGroupLabel>
          <CreateFeedModal />
          <SidebarGroupContent>
            <SidebarMenu>
              <FeedList />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Collections</SidebarGroupLabel>
          <CreateCollectionModal />
          <SidebarGroupContent>
            <SidebarMenu>
              <CollectionList />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
      <SidebarFooter>
        <UserCard user={props.user} />
      </SidebarFooter>
    </Sidebar>
  )
}
