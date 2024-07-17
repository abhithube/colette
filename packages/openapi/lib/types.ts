import type { components, operations } from './openapi'

type schemas = components['schemas']

export type Entry = schemas['Entry']
export type Feed = schemas['Feed']
export type Profile = schemas['Profile']

export type EntryList = schemas['EntryList']
export type FeedList = schemas['FeedList']
export type ProfileList = schemas['ProfileList']

export type CreateFeed = schemas['CreateFeed']
export type CreateProfile = schemas['CreateProfile']
export type Login = schemas['Login']

export type ListEntriesQuery = NonNullable<
	operations['listEntries']['parameters']['query']
>
