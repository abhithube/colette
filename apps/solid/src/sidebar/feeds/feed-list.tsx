import { FeedItem } from './feed-item'
import { listFeedsOptions } from '@colette/query'
import { createQuery } from '@tanstack/solid-query'
import { type Component, For, Show } from 'solid-js'
import { useAPI } from '~/lib/api-context'

export const FeedList: Component = () => {
  const api = useAPI()

  const query = createQuery(() => listFeedsOptions({}, api))

  return (
    <Show when={query.data}>
      {(feeds) => (
        <For each={feeds().data}>{(item) => <FeedItem feed={item} />}</For>
      )}
    </Show>
  )
}
