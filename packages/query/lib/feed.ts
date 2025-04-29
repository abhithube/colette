import type { API, FeedDetect, FeedScrape } from '@colette/core'
import { useMutation } from '@tanstack/react-query'

export const useDetectFeedsMutation = (api: API) => {
  return useMutation({
    mutationFn: (data: FeedDetect) => api.feeds.detectFeeds(data),
  })
}

export const useScrapeFeedMutation = (api: API) => {
  return useMutation({
    mutationFn: (data: FeedScrape) => api.feeds.scrapeFeed(data),
  })
}
