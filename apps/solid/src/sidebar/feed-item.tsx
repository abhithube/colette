import type { Feed } from '@colette/core'
import { A } from '@solidjs/router'
import type { Component } from 'solid-js'
import { Favicon } from '~/components/favicon'
import { SidebarMenuButton, SidebarMenuItem } from '~/components/ui/sidebar'

export const FeedItem: Component<{ feed: Feed }> = (props) => {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton as={A} href={`/feeds/${props.feed.id}`}>
        <Favicon url={props.feed.link} />
        {props.feed.title ?? props.feed.originalTitle}
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}
