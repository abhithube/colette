import {
  type ApiClient,
  Paginated_SubscriptionEntry,
  get_ListSubscriptionEntries,
} from './openapi.gen'
import type { z } from 'zod'

export const SubscriptionEntryListQuery =
  get_ListSubscriptionEntries.parameters.shape.query
export type SubscriptionEntryListQuery = z.infer<
  typeof SubscriptionEntryListQuery
>

export type SubscriptionEntryList = Paginated_SubscriptionEntry
export const SubscriptionEntryList = Paginated_SubscriptionEntry

export interface SubscriptionEntryAPI {
  listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryList>
}

export class HTTPSubscriptionEntryAPI implements SubscriptionEntryAPI {
  constructor(private client: ApiClient) {}

  listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryList> {
    return this.client
      .get('/subscriptionEntries', {
        query: SubscriptionEntryListQuery.parse(query),
      })
      .then(SubscriptionEntryList.parse)
  }
}
