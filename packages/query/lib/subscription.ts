import { SUBSCRIPTION_ENTRIES_PREFIX } from './subscription-entry'
import type {
  API,
  SubscriptionCreate,
  SubscriptionListQuery,
  SubscriptionUpdate,
} from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

export const SUBSCRIPTIONS_PREFIX = 'subscriptions'

export const listSubscriptionsOptions = (
  api: API,
  query: SubscriptionListQuery = {},
) =>
  queryOptions({
    queryKey: [SUBSCRIPTIONS_PREFIX, query],
    queryFn: () => api.subscriptions.listSubscriptions(query),
  })

export const getSubscriptionOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [SUBSCRIPTIONS_PREFIX, id],
    queryFn: () => api.subscriptions.getSubscription(id),
  })

export const useCreateSubscriptionMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: SubscriptionCreate) =>
      api.subscriptions.createSubscription(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useUpdateSubscriptionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: SubscriptionUpdate) =>
      api.subscriptions.updateSubscription(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useDeleteSubscriptionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.subscriptions.deleteSubscription(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useMarkSubscriptionEntryAsReadMutation = (
  sid: string,
  eid: string,
) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.subscriptions.markSubscriptionEntryAsRead(sid, eid),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTION_ENTRIES_PREFIX],
      })
    },
  })
}

export const useMarkSubscriptionEntryAsUnreadMutation = (
  sid: string,
  eid: string,
) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.subscriptions.markSubscriptionEntryAsUnread(sid, eid),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTION_ENTRIES_PREFIX],
      })
    },
  })
}
