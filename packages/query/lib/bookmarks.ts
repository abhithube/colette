import type {
	API,
	Bookmark,
	BookmarkCreate,
	ListBookmarksQuery,
} from '@colette/openapi'
import {
	type UseMutationOptions,
	infiniteQueryOptions,
} from '@tanstack/react-query'

export const listBookmarksOptions = (
	query: ListBookmarksQuery,
	profileId: string,
	api: API,
) =>
	infiniteQueryOptions({
		queryKey: ['profiles', profileId, 'bookmarks', query],
		queryFn: ({ pageParam, signal }) =>
			api.bookmarks.list(
				{
					...query,
					publishedAt: pageParam,
				},
				{
					signal,
				},
			),
		initialPageParam: undefined as string | undefined,
		getNextPageParam: (lastPage) => {
			return lastPage.hasMore
				? lastPage.data[lastPage.data.length - 1].publishedAt
				: undefined
		},
	})

export const createBookmarkOptions = (
	options: Omit<
		UseMutationOptions<Bookmark, Error, BookmarkCreate>,
		'mutationFn'
	>,
	api: API,
) => {
	return {
		...options,
		mutationFn: (body) => api.bookmarks.create(body),
	} as UseMutationOptions<Bookmark, Error, BookmarkCreate>
}
