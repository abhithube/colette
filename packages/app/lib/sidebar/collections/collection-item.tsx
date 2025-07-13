import type { Collection } from '@colette/core/types'
import { Link } from '@colette/router'
import { Sidebar } from '@colette/ui'
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
