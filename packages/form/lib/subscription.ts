import type { Feed, FeedDetected, Subscription, Tag } from '@colette/core/types'
import { formOptions } from '@tanstack/react-form'
import { z } from 'zod'

export const DETECT_FEEDS_FORM = 'detect-feeds'

export const detectFeedsFormOptions = () =>
  formOptions({
    defaultValues: {
      url: '',
    },
    validators: {
      onBlur: z.object({
        url: z.string().url('URL is not valid'),
      }),
    },
  })

export const SCRAPE_FEED_FORM = 'scrape-feed'

export const scrapeFeedFormOptions = (initial: FeedDetected[]) =>
  formOptions({
    defaultValues: {
      url: initial[0].url,
    },
  })

export const CREATE_SUBSCRIPTION_FORM = 'create-subscription'

export const createSubscriptionFormOptions = (initial: Feed) =>
  formOptions({
    defaultValues: {
      title: initial.title,
      description: initial.description,
    },
    validators: {
      onBlur: z.object({
        title: z.string().min(1, "Title can't be empty"),
        description: z.string().nullable(),
      }),
    },
  })

export const UPDATE_SUBSCRIPTION_FORM = 'update-subscription'

export const updateSubscriptionFormOptions = (initial: Subscription) =>
  formOptions({
    defaultValues: {
      title: initial.title,
      description: initial.description,
    },
    validators: {
      onBlur: z.object({
        title: z.string().min(1, "Title can't be empty"),
        description: z.string().nullable(),
      }),
    },
  })

export const LINK_SUBSCRIPTION_TAGS_FORM = 'link-subscription-tags'

export const linkSubscriptionTagsFormOptions = (initial?: Tag[]) =>
  formOptions({
    defaultValues: {
      tagIds: initial?.map((tag) => tag.id) ?? [],
    },
    validators: {
      onBlur: z.object({
        tagIds: z.array(z.string().uuid('UUID is not valid')),
      }),
    },
  })

export const IMPORT_SUBSCRIPTIONS_FORM = 'import-subscription'

export const importSubscriptionsFormOptions = () =>
  formOptions({
    defaultValues: {
      file: undefined as unknown as File,
    },
  })
