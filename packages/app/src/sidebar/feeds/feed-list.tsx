import { listFeedsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import type { FC } from 'react'
import { useAPI } from '../../lib/api-context'
import { FeedItem } from './feed-item'

export const FeedList: FC = () => {
  const api = useAPI()

  const { data: feeds, isLoading } = useQuery(listFeedsOptions({}, api))

  if (isLoading || !feeds) return

  return feeds.data.map((feed) => <FeedItem key={feed.id} feed={feed} />)
}
