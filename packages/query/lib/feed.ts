import type { FeedDetect, FeedScrape } from '@colette/core'
import { useAPI } from '@colette/util'
import { useMutation } from '@tanstack/react-query'

export const useDetectFeedsMutation = () => {
  const api = useAPI()

  return useMutation({
    mutationFn: (data: FeedDetect) => api.feeds.detectFeeds(data),
  })
}

export const useScrapeFeedMutation = () => {
  const api = useAPI()

  return useMutation({
    mutationFn: (data: FeedScrape) => api.feeds.scrapeFeed(data),
  })
}
