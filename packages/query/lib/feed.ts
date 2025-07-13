import { detectFeeds, scrapeFeed } from '@colette/core/http'
import type { FeedDetect, FeedScrape } from '@colette/core/types'
import { useMutation } from '@tanstack/react-query'

export const useDetectFeedsMutation = () => {
  return useMutation({
    mutationFn: (data: FeedDetect) => detectFeeds(data),
  })
}

export const useScrapeFeedMutation = () => {
  return useMutation({
    mutationFn: (data: FeedScrape) => scrapeFeed(data),
  })
}
