import { components, operations, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type SubscriptionEntry = components['schemas']['SubscriptionEntry']
export type SubscriptionEntryDetails =
  components['schemas']['SubscriptionEntryDetails']
export type SubscriptionEntryDetailsList =
  components['schemas']['Paginated_SubscriptionEntryDetails']

export type SubscriptionEntryListQuery = NonNullable<
  operations['listSubscriptionEntries']['parameters']['query']
>

export interface SubscriptionEntryAPI {
  listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryDetailsList>
}

export class HTTPSubscriptionEntryAPI implements SubscriptionEntryAPI {
  constructor(private client: Client<paths>) {}

  async listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryDetailsList> {
    const res = await this.client.GET('/subscriptionEntries', {
      params: {
        query,
      },
    })

    return res.data!
  }
}
