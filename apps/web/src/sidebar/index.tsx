import { CollectionFolderContents } from './collections/collection-folder-contents'
import { CreateCollectionModal } from './collections/create-collection-modal'
import { CreateFeedModal } from './feeds/create-form/create-feed-modal'
import { FeedFolderContents } from './feeds/feed-folder-contents'
import { UserCard } from './user-card'
import type { User } from '@colette/core'
import { Archive, History, Home } from 'lucide-react'
import type { FC } from 'react'
import { Link } from 'wouter'
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
              <FeedFolderContents />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Collections</SidebarGroupLabel>
          <CreateCollectionModal />
          <SidebarGroupContent>
            <SidebarMenu>
              <CollectionFolderContents />
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
