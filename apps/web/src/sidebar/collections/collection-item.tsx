import { Collection } from '@colette/core'
import { Sidebar } from '@colette/ui'
import { Link } from '@tanstack/react-router'
import { Library } from 'lucide-react'

export const CollectionItem = (props: { collection: Collection }) => {
  return (
    <Sidebar.MenuItem>
      <Sidebar.MenuButton asChild>
        <Link
          to="/collections/$collectionId"
          params={{
            collectionId: props.collection.id,
          }}
        >
          <Library />
          {props.collection.title}
        </Link>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
  )
}
