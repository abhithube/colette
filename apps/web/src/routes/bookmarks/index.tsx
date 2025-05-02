import { BookmarkGrid } from './components/bookmark-grid'
import { CreateBookmarkModal } from './components/create-form/create-bookmark-modal'
import { listBookmarksOptions } from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useInfiniteQuery } from '@tanstack/react-query'
import { Plus } from 'lucide-react'
import { useEffect } from 'react'

export const StashPage = () => {
  const api = useAPI()

  const query = useInfiniteQuery(listBookmarksOptions(api))

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
