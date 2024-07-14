import type { components } from './openapi'

type schemas = components['schemas']

export type Entry = schemas['Entry']
export type Feed = schemas['Feed']
export type FeedList = schemas['FeedList']
export type Profile = schemas['Profile']

export type CreateFeed = schemas['CreateFeed']
export type Login = schemas['Login']

export type ValidationError = schemas['ValidationError']
