import { listFeedsOptions } from '@colette/solid-query'
import { createQuery } from '@tanstack/solid-query'
import { type Component, For, Show } from 'solid-js'
import { useAPI } from '~/lib/api-context'
import { FeedItem } from './feed-item'

export const FeedList: Component = () => {
  const query = createQuery(() => listFeedsOptions({}, useAPI()))

  return (
    <Show when={query.data}>
      {(feeds) => (
        <For each={feeds().data}>{(item) => <FeedItem feed={item} />}</For>
      )}
    </Show>
  )
}
