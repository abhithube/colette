import { Stream } from '@colette/core'
import { Sidebar } from '@colette/ui'
import { Podcast } from 'lucide-react'
import { Link } from 'wouter'

export const StreamItem = (props: { stream: Stream }) => {
  return (
    <Sidebar.MenuItem>
      <Sidebar.MenuButton asChild>
        <Link to={`/streams/${props.stream.id}`}>
          <Podcast />
          {props.stream.title}
        </Link>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
  )
}
