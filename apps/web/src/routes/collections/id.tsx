import { BookmarkGrid } from '../bookmarks/components/bookmark-grid'
import { getCollectionOptions, listBookmarksOptions } from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { getRouteApi } from '@tanstack/react-router'
import { Pencil, Trash2 } from 'lucide-react'
import { useEffect } from 'react'

const routeApi = getRouteApi('/layout/collections/$collectionId')

export const CollectionPage = () => {
  const context = routeApi.useRouteContext()
  const params = routeApi.useParams()

  const collectionQuery = useQuery(
    getCollectionOptions(context.api, params.collectionId),
  )
  const bookmarksQuery = useInfiniteQuery(
    listBookmarksOptions(context.api, {
      collectionId: params.collectionId,
    }),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [params.collectionId])

  if (!collectionQuery.data) return

  if (
    collectionQuery.isLoading ||
    !collectionQuery.data ||
    bookmarksQuery.isLoading ||
    !bookmarksQuery.data
  )
    return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="line-clamp-1 text-3xl font-medium">
          {collectionQuery.data.title}
        </h1>
        <div className="flex gap-2">
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="secondary">
                <Pencil />
                Edit
              </Button>
            </Dialog.Trigger>
          </Dialog.Root>
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="destructive">
                <Trash2 />
                Delete
              </Button>
            </Dialog.Trigger>
          </Dialog.Root>
        </div>
      </div>
      <main>
        <BookmarkGrid
          bookmarks={bookmarksQuery.data.pages.flatMap((page) => page.data)}
          hasMore={bookmarksQuery.hasNextPage}
          fetchMore={bookmarksQuery.fetchNextPage}
        />
      </main>
    </>
  )
}
