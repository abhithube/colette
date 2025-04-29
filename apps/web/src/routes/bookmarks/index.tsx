import { BookmarkGrid } from './components/bookmark-grid'
import { CreateBookmarkModal } from './components/create-form/create-bookmark-modal'
import { listBookmarksOptions } from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useInfiniteQuery } from '@tanstack/react-query'
import { getRouteApi } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useEffect } from 'react'

const routeApi = getRouteApi('/layout/stash')

export const StashPage = () => {
  const context = routeApi.useRouteContext()

  const query = useInfiniteQuery(listBookmarksOptions(context.api))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (query.isLoading || !query.data) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">Stash</h1>
        <Dialog.Root>
          <Dialog.Trigger asChild>
            <Button variant="secondary">
              <Plus />
              New
            </Button>
          </Dialog.Trigger>
          <Dialog.Context>
            {(dialogProps) => (
              <CreateBookmarkModal close={() => dialogProps.setOpen(false)} />
            )}
          </Dialog.Context>
        </Dialog.Root>
      </div>
      <main>
        <BookmarkGrid
          bookmarks={query.data.pages.flatMap((page) => page.data)}
          hasMore={query.hasNextPage}
          fetchMore={query.fetchNextPage}
        />
      </main>
    </>
  )
}
