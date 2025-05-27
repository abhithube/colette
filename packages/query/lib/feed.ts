import {
  detectFeeds,
  scrapeFeed,
  type FeedDetect,
  type FeedScrape,
} from '@colette/core'
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
