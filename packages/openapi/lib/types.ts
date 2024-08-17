import type createClient from 'openapi-fetch'
import type { paths } from './openapi'

export type Client = ReturnType<typeof createClient<paths>>
