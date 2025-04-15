import {
  type ApiClient,
  get_GetSubscription,
  get_ListSubscriptions,
  LinkSubscriptionTags,
  Paginated_SubscriptionDetails,
  Subscription,
  SubscriptionCreate,
  SubscriptionDetails,
  SubscriptionEntry,
  SubscriptionUpdate,
} from './openapi.gen'
import type { z } from 'zod'

export const SubscriptionListQuery =
  get_ListSubscriptions.parameters.shape.query
export type SubscriptionListQuery = z.infer<typeof SubscriptionListQuery>

export const SubscriptionGetQuery = get_GetSubscription.parameters.shape.query
export type SubscriptionGetQuery = z.infer<typeof SubscriptionGetQuery>

export type SubscriptionDetailsList = Paginated_SubscriptionDetails
export const SubscriptionDetailsList = Paginated_SubscriptionDetails

export interface SubscriptionAPI {
  listSubscriptions(
    query: SubscriptionListQuery,
  ): Promise<SubscriptionDetailsList>

  getSubscription(
    id: string,
    query: SubscriptionGetQuery,
  ): Promise<SubscriptionDetails>

  createSubscription(data: SubscriptionCreate): Promise<Subscription>

  updateSubscription(
    id: string,
    data: SubscriptionUpdate,
  ): Promise<Subscription>

  deleteSubscription(id: string): Promise<void>

  linkSubscriptionTags(id: string, data: LinkSubscriptionTags): Promise<void>

  markSubscriptionEntryAsRead(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry>

  markSubscriptionEntryAsUnread(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry>

  importSubscriptions(data: File): Promise<void>
}

export class HTTPSubscriptionAPI implements SubscriptionAPI {
  constructor(private client: ApiClient) {}

  listSubscriptions(
    query: SubscriptionListQuery,
  ): Promise<SubscriptionDetailsList> {
    return this.client
      .get('/subscriptions', {
        query: SubscriptionListQuery.parse(query),
      })
      .then(SubscriptionDetailsList.parse)
  }

  getSubscription(
    id: string,
    query: SubscriptionGetQuery,
  ): Promise<SubscriptionDetails> {
    return this.client
      .get('/subscriptions/{id}', {
        path: {
          id,
        },
        query: SubscriptionGetQuery.parse(query),
      })
      .then(SubscriptionDetails.parse)
  }

  createSubscription(data: SubscriptionCreate): Promise<Subscription> {
    return this.client
      .post('/subscriptions', {
        body: SubscriptionCreate.parse(data),
      })
      .then(Subscription.parse)
  }

  updateSubscription(
    id: string,
    data: SubscriptionUpdate,
  ): Promise<Subscription> {
    return this.client
      .patch('/subscriptions/{id}', {
        path: {
          id,
        },
        body: SubscriptionUpdate.parse(data),
      })
      .then(Subscription.parse)
  }

  deleteSubscription(id: string): Promise<void> {
    return this.client
      .delete('/subscriptions/{id}', {
        path: {
          id,
        },
      })
      .then()
  }

  linkSubscriptionTags(id: string, data: LinkSubscriptionTags): Promise<void> {
    return this.client
      .patch('/subscriptions/{id}/linkTags', {
        path: {
          id,
        },
        body: LinkSubscriptionTags.parse(data),
      })
      .then()
  }

  markSubscriptionEntryAsRead(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry> {
    return this.client
      .post('/subscriptions/{sid}/entries/{eid}/markAsRead', {
        path: {
          sid,
          eid,
        },
      })
      .then(SubscriptionEntry.parse)
  }

  markSubscriptionEntryAsUnread(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry> {
    return this.client
      .post('/subscriptions/{sid}/entries/{eid}/markAsUnread', {
        path: {
          sid,
          eid,
        },
      })
      .then(SubscriptionEntry.parse)
  }

  async importSubscriptions(data: File): Promise<void> {
    return this.client
      .post('/subscriptions/import', {
        body: new Uint8Array(await data.arrayBuffer()),
        header: {
          'Content-Type': 'application/octet-stream',
        },
      } as any)
      .then()
  }
}
