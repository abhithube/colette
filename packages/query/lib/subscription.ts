import { SUBSCRIPTION_ENTRIES_PREFIX } from './subscription-entry'
import {
  createSubscription,
  deleteSubscription,
  getSubscription,
  importSubscriptions,
  linkSubscriptionTags,
  listSubscriptions,
  markSubscriptionEntryAsRead,
  markSubscriptionEntryAsUnread,
  updateSubscription,
} from '@colette/core/http'
import type {
  GetSubscriptionQueryParams,
  LinkSubscriptionTags,
  ListSubscriptionsQueryParams,
  SubscriptionCreate,
  SubscriptionUpdate,
} from '@colette/core/types'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const SUBSCRIPTIONS_PREFIX = 'subscriptions'

export const listSubscriptionsOptions = (
  query: ListSubscriptionsQueryParams = {},
) =>
  queryOptions({
    queryKey: [SUBSCRIPTIONS_PREFIX, query],
    queryFn: () => listSubscriptions(query),
  })

export const getSubscriptionOptions = (
  id: string,
  query: GetSubscriptionQueryParams = {},
) =>
  queryOptions({
    queryKey: [SUBSCRIPTIONS_PREFIX, id],
    queryFn: () => getSubscription(id, query),
  })

export const useCreateSubscriptionMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: SubscriptionCreate) => createSubscription(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useUpdateSubscriptionMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: SubscriptionUpdate) => updateSubscription(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useDeleteSubscriptionMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteSubscription(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}

export const useLinkSubscriptionTagsMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: LinkSubscriptionTags) => linkSubscriptionTags(id, data),
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
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => markSubscriptionEntryAsRead(sid, eid),
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
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => markSubscriptionEntryAsUnread(sid, eid),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTION_ENTRIES_PREFIX],
      })
    },
  })
}

export const useImportSubscriptionsMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (data: File) =>
      importSubscriptions(new Uint8Array(await data.arrayBuffer()) as never),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}
