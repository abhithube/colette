import { FeedItem } from './feed-item'
import { listFeedsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const FeedList: FC = () => {
  const api = useAPI()

  const { data: feeds, isLoading } = useQuery(listFeedsOptions(api))

  if (isLoading || !feeds) return

  return (
    <div className="flex flex-col gap-4 px-8">
      {feeds.data.map((feed) => (
        <FeedItem key={feed.id} feed={feed} />
      ))}
    </div>
  )
}
