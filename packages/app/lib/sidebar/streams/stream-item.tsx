import type { Stream } from '@colette/core/types'
import { Link } from '@colette/router'
import { Sidebar } from '@colette/ui'
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
