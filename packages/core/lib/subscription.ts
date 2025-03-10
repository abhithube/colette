import {
  type ApiClient,
  Subscription,
  SubscriptionCreate,
  SubscriptionUpdate,
  Paginated_Subscription,
  get_ListSubscriptions,
  SubscriptionEntry,
} from './openapi.gen'
import type { z } from 'zod'

export const SubscriptionListQuery =
  get_ListSubscriptions.parameters.shape.query
export type SubscriptionListQuery = z.infer<typeof SubscriptionListQuery>

export type SubscriptionList = Paginated_Subscription
export const SubscriptionList = Paginated_Subscription

export interface SubscriptionAPI {
  listSubscriptions(query: SubscriptionListQuery): Promise<SubscriptionList>

  getSubscription(id: string): Promise<Subscription>

  createSubscription(data: SubscriptionCreate): Promise<Subscription>

  updateSubscription(
    id: string,
    data: SubscriptionUpdate,
  ): Promise<Subscription>

  deleteSubscription(id: string): Promise<void>

  markSubscriptionEntryAsRead(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry>

  markSubscriptionEntryAsUnread(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry>
}

export class HTTPSubscriptionAPI implements SubscriptionAPI {
  constructor(private client: ApiClient) {}

  listSubscriptions(query: SubscriptionListQuery): Promise<SubscriptionList> {
    return this.client
      .get('/subscriptions', {
        query: SubscriptionListQuery.parse(query),
      })
      .then(SubscriptionList.parse)
  }

  getSubscription(id: string): Promise<Subscription> {
    return this.client
      .get('/subscriptions/{id}', {
        path: {
          id,
        },
      })
      .then(Subscription.parse)
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
}
