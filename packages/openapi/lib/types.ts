import type { components, operations } from './openapi'

type schemas = components['schemas']

export type Bookmark = schemas['Bookmark']
export type Feed = schemas['Feed']
export type FeedEntry = schemas['FeedEntry']
export type File = schemas['File']
export type Profile = schemas['Profile']
export type Tag = schemas['Tag']
export type User = schemas['User']

export type BookmarkList = schemas['BookmarkList']
export type FeedList = schemas['FeedList']
export type FeedEntryList = schemas['FeedEntryList']
export type ProfileList = schemas['ProfileList']
export type TagList = schemas['TagList']

export type BookmarkCreate = schemas['BookmarkCreate']
export type FeedCreate = schemas['FeedCreate']
export type ProfileCreate = schemas['ProfileCreate']
export type Login = schemas['Login']
export type Register = schemas['Register']
export type TagCreate = schemas['TagCreate']

export type BookmarkUpdate = schemas['BookmarkUpdate']
export type FeedUpdate = schemas['FeedUpdate']
export type FeedEntryUpdate = schemas['FeedEntryUpdate']
export type ProfileUpdate = schemas['ProfileUpdate']
export type TagUpdate = schemas['TagUpdate']

export type ListBookmarksQuery = NonNullable<
  operations['listBookmarks']['parameters']['query']
>
export type ListFeedsQuery = NonNullable<
  operations['listFeeds']['parameters']['query']
>
export type ListFeedEntriesQuery = NonNullable<
  operations['listFeedEntries']['parameters']['query']
>
export type ListTagsQuery = NonNullable<
  operations['listTags']['parameters']['query']
>
