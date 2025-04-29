import { Stream } from '@colette/core'
import { Sidebar } from '@colette/ui'
import { Link } from '@tanstack/react-router'
import { Podcast } from 'lucide-react'

export const StreamItem = (props: { stream: Stream }) => {
  return (
    <Sidebar.MenuItem>
      <Sidebar.MenuButton asChild>
        <Link
          to="/streams/$streamId"
          params={{
            streamId: props.stream.id,
          }}
        >
          <Podcast />
          {props.stream.title}
        </Link>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
  )
}
