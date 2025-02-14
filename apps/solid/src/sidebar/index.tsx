import { CollectionList } from './collections/collection-list'
import { CreateCollectionModal } from './collections/create-collection-modal'
import { CreateFeedModal } from './feeds/create-form/create-feed-modal'
import { FeedList } from './feeds/feed-list'
import { UserCard } from './user-card'
import type { User } from '@colette/core'
import { A } from '@solidjs/router'
import type { Component } from 'solid-js'
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

export const AppSidebar: Component<{ user: User }> = ({ user }) => {
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
                <SidebarMenuButton as={A} href="/">
                  Home
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton as={A} href="/archived">
                  Archived
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
        <UserCard user={user} />
      </SidebarFooter>
    </Sidebar>
  )
}
