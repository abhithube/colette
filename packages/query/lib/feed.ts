import type { FeedDetect } from '@colette/core'
import { useAPI } from '@colette/util'
import { useMutation } from '@tanstack/react-query'

export const useDetectFeedsMutation = () => {
  const api = useAPI()

  return useMutation({
    mutationFn: (data: FeedDetect) => api.feeds.detectFeeds(data),
  })
}
