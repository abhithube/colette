import {
  type ApiClient,
  Paginated_SubscriptionEntryDetails,
  get_ListSubscriptionEntries,
} from './openapi.gen'
import type { z } from 'zod'

export const SubscriptionEntryListQuery =
  get_ListSubscriptionEntries.parameters.shape.query
export type SubscriptionEntryListQuery = z.infer<
  typeof SubscriptionEntryListQuery
>

export type SubscriptionEntryDetailsList = Paginated_SubscriptionEntryDetails
export const SubscriptionEntryDetailsList = Paginated_SubscriptionEntryDetails

export interface SubscriptionEntryAPI {
  listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryDetailsList>
}

export class HTTPSubscriptionEntryAPI implements SubscriptionEntryAPI {
  constructor(private client: ApiClient) {}

  listSubscriptionEntries(
    query: SubscriptionEntryListQuery,
  ): Promise<SubscriptionEntryDetailsList> {
    return this.client
      .get('/subscriptionEntries', {
        query: SubscriptionEntryListQuery.parse(query),
      })
      .then(SubscriptionEntryDetailsList.parse)
  }
}
