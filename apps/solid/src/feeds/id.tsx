import { getFeedOptions } from '@colette/solid-query'
import { useParams } from '@solidjs/router'
import { createQuery } from '@tanstack/solid-query'
import CircleX from 'lucide-solid/icons/circle-x'
import ExternalLink from 'lucide-solid/icons/external-link'
import ListChecks from 'lucide-solid/icons/list-checks'
import Pencil from 'lucide-solid/icons/pencil'
import { type Component, Show } from 'solid-js'
import { Button } from '~/components/ui/button'
import { useAPI } from '~/lib/api-context'

export const FeedPage: Component = () => {
  const params = useParams<{ id: string }>()

  const query = createQuery(() => getFeedOptions(params.id, useAPI()))

  return (
    <Show when={query.data}>
      {(feed) => (
        <>
          <div class="sticky top-0 z-10 flex justify-between bg-background p-8">
            <h1 class="font-medium text-3xl">
              {feed().title ?? feed().originalTitle}
            </h1>
            <div class="flex gap-2">
              <Button
                as="a"
                variant="secondary"
                href={feed().link}
                target="_blank"
              >
                <ExternalLink />
                Open Link
              </Button>
              <Button variant="secondary">
                <Pencil />
                Edit
              </Button>
              <Button variant="secondary">
                <ListChecks />
                Mark as Read
              </Button>
              <Button variant="destructive">
                <CircleX />
                Unsubscribe
              </Button>
            </div>
          </div>
        </>
      )}
    </Show>
  )
}
