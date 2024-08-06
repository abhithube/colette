import type { API, ListTagsQuery } from '@colette/openapi'
import { queryOptions } from '@tanstack/react-query'

export const listTagsOptions = (
	query: ListTagsQuery,
	profileId: string,
	api: API,
) =>
	queryOptions({
		queryKey: ['profiles', profileId, 'tags', query],
		queryFn: ({ signal }) =>
			api.tags.list(query, {
				signal,
			}),
	})

export const getTagOptions = (id: string, api: API) =>
	queryOptions({
		queryKey: ['feeds', id],
		queryFn: ({ signal }) =>
			api.tags.get(id, {
				signal,
			}),
	})
