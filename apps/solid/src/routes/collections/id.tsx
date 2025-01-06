import { getCollectionOptions } from '@colette/query'
import { useParams } from '@solidjs/router'
import { createQuery } from '@tanstack/solid-query'
import BookmarkPlus from 'lucide-solid/icons/bookmark-plus'
import CircleX from 'lucide-solid/icons/circle-x'
import Pencil from 'lucide-solid/icons/pencil'
import { type Component, Show } from 'solid-js'
import { Button } from '~/components/ui/button'
import { useAPI } from '~/lib/api-context'
import { BookmarkGrid } from './bookmark-grid'

export const CollectionPage: Component = () => {
  const params = useParams<{ id: string }>()

  const query = createQuery(() => getCollectionOptions(params.id, useAPI()))

  return (
    <Show when={query.data}>
      {(collection) => (
        <>
          <div class="sticky top-0 z-10 flex justify-between bg-background p-8">
            <h1 class="font-medium text-3xl">{collection().title}</h1>
            <div class="flex gap-2">
              <Button variant="secondary">
                <Pencil />
                Edit
              </Button>
              <Button variant="secondary">
                <BookmarkPlus />
                Add
              </Button>
              <Button variant="destructive">
                <CircleX />
                Delete
              </Button>
            </div>
          </div>
          <main>
            <BookmarkGrid collectionId={collection().id} />
          </main>
        </>
      )}
    </Show>
  )
}
