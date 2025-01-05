import { listCollectionsOptions } from '@colette/solid-query'
import { createQuery } from '@tanstack/solid-query'
import { type Component, For, Show } from 'solid-js'
import { useAPI } from '~/lib/api-context'
import { CollectionItem } from './collection-item'

export const CollectionList: Component = () => {
  const api = useAPI()

  const query = createQuery(() => listCollectionsOptions(api))

  return (
    <Show when={query.data}>
      {(collections) => (
        <For each={collections().data}>
          {(item) => <CollectionItem collection={item} />}
        </For>
      )}
    </Show>
  )
}
