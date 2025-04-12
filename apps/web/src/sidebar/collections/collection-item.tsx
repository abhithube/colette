import { Collection } from '@colette/core'
import { Sidebar } from '@colette/ui'
import { Library } from 'lucide-react'
import { Link } from 'wouter'

export const CollectionItem = (props: { collection: Collection }) => {
  return (
    <Sidebar.MenuItem>
      <Sidebar.MenuButton asChild>
        <Link to={`/collections/${props.collection.id}`}>
          <Library />
          {props.collection.title}
        </Link>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
  )
}
