import type {
  InfiniteData,
  InfiniteQueryObserverOptions,
  MutationOptions,
  QueryKey,
  QueryOptions,
} from '@tanstack/query-core'

export type BaseQueryOptions<
  TQueryFnData = unknown,
  TData = TQueryFnData,
> = QueryOptions<TQueryFnData, Error, TData> & {
  queryKey: QueryKey
  initialData?: undefined
}

export type BaseInfiniteQueryOptions<
  TQueryFnData = unknown,
  TPageParam = unknown,
> = InfiniteQueryObserverOptions<
  TQueryFnData,
  Error,
  InfiniteData<TQueryFnData>,
  TQueryFnData,
  QueryKey,
  TPageParam
> & {
  initialData?: undefined
}

export type BaseMutationOptions<
  TQueryFnData = unknown,
  TVariables = void,
> = MutationOptions<TQueryFnData, Error, TVariables>
