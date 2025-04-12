import { EntryList } from '../subscriptions/components/entry-list'
import {
  getStreamOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { Pencil, Trash2 } from 'lucide-react'
import { useEffect } from 'react'
import { useParams } from 'wouter'

export const StreamPage = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const streamQuery = useQuery(getStreamOptions(api, id))
  const entriesQuery = useInfiniteQuery(
    listSubscriptionEntriesOptions(api, {
      streamId: id,
    }),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (
    streamQuery.isLoading ||
    !streamQuery.data ||
    entriesQuery.isLoading ||
    !entriesQuery.data
  )
    return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="line-clamp-1 text-3xl font-medium">
          {streamQuery.data.title}
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
        <EntryList
          entries={entriesQuery.data.pages.flatMap((page) => page.data)}
          hasMore={entriesQuery.hasNextPage}
          fetchMore={entriesQuery.fetchNextPage}
        />
      </main>
    </>
  )
}
