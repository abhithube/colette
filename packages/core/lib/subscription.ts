import { components, operations, paths } from './openapi'
import { SubscriptionEntry } from './subscription-entry'
import { Client } from 'openapi-fetch'

export type Subscription = components['schemas']['Subscription']
export type SubscriptionDetails = components['schemas']['SubscriptionDetails']
export type SubscriptionDetailsList =
  components['schemas']['Paginated_SubscriptionDetails']
export type SubscriptionCreate = components['schemas']['SubscriptionCreate']
export type SubscriptionUpdate = components['schemas']['SubscriptionUpdate']
export type LinkSubscriptionTags = components['schemas']['LinkSubscriptionTags']

export type SubscriptionListQuery = NonNullable<
  operations['listSubscriptions']['parameters']['query']
>
export type SubscriptionGetQuery = NonNullable<
  operations['getSubscription']['parameters']['query']
>

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
  constructor(private client: Client<paths>) {}

  async listSubscriptions(
    query: SubscriptionListQuery,
  ): Promise<SubscriptionDetailsList> {
    const res = await this.client.GET('/subscriptions', {
      params: {
        query,
      },
    })

    return res.data!
  }

  async getSubscription(
    id: string,
    query: SubscriptionGetQuery,
  ): Promise<SubscriptionDetails> {
    const res = await this.client.GET('/subscriptions/{id}', {
      params: {
        path: {
          id,
        },
        query,
      },
    })

    return res.data!
  }

  async createSubscription(data: SubscriptionCreate): Promise<Subscription> {
    const res = await this.client.POST('/subscriptions', {
      body: data,
    })

    return res.data!
  }

  async updateSubscription(
    id: string,
    data: SubscriptionUpdate,
  ): Promise<Subscription> {
    const res = await this.client.PATCH('/subscriptions/{id}', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })

    return res.data!
  }

  async deleteSubscription(id: string): Promise<void> {
    await this.client.DELETE('/subscriptions/{id}', {
      params: {
        path: {
          id,
        },
      },
    })
  }

  async linkSubscriptionTags(
    id: string,
    data: LinkSubscriptionTags,
  ): Promise<void> {
    await this.client.PATCH('/subscriptions/{id}/linkTags', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })
  }

  async markSubscriptionEntryAsRead(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry> {
    const res = await this.client.POST(
      '/subscriptions/{sid}/entries/{eid}/markAsRead',
      {
        params: {
          path: {
            sid,
            eid,
          },
        },
      },
    )

    return res.data!
  }

  async markSubscriptionEntryAsUnread(
    sid: string,
    eid: string,
  ): Promise<SubscriptionEntry> {
    const res = await this.client.POST(
      '/subscriptions/{sid}/entries/{eid}/markAsUnread',
      {
        params: {
          path: {
            sid,
            eid,
          },
        },
      },
    )

    return res.data!
  }

  async importSubscriptions(data: File): Promise<void> {
    await this.client.POST('/subscriptions/import', {
      body: new Uint8Array(await data.arrayBuffer()) as any,
      header: {
        'Content-Type': 'application/octet-stream',
      },
    })
  }
}
