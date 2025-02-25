import { FeedItem } from './feed-item'
import { listFeedsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const FeedList: FC = () => {
  const api = useAPI()

  const query = useQuery(listFeedsOptions(api))

  if (query.isLoading || !query.data) return

  return (
    <div className="flex flex-col gap-4 px-8">
      {query.data.data.map((feed) => (
        <FeedItem key={feed.id} feed={feed} />
      ))}
    </div>
  )
}
