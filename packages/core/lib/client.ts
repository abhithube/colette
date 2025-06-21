import { refreshToken } from './gen/http'
import { ApiError } from './gen/types'

export type RequestConfig<TData = unknown> = {
  baseURL?: string
  url?: string
  method?: 'GET' | 'PUT' | 'PATCH' | 'POST' | 'DELETE' | 'OPTIONS' | 'HEAD'
  params?: unknown
  data?: TData | FormData
  responseType?:
    | 'arraybuffer'
    | 'blob'
    | 'document'
    | 'json'
    | 'text'
    | 'stream'
  signal?: AbortSignal
  headers?: Headers | Record<string, string>
  accessToken?: string
}

export type ResponseConfig<TData = unknown> = {
  data: TData
  status: number
  statusText: string
  headers: Headers
}

export type ResponseErrorConfig<TError = unknown> = TError

let _config: Partial<RequestConfig> = {}

const getConfig = () => _config

const setConfig = (config: Partial<RequestConfig>) => {
  _config = config
  return getConfig()
}

const handleRefresh = async (): Promise<string | null> => {
  try {
    const data = await refreshToken()

    setConfig({
      ...getConfig(),
      accessToken: data.accessToken,
    })

    return data.accessToken
  } catch {
    return null
  }
}

let _refreshPromise: Promise<string | null> | null = null

const getOrHandleRefresh = async (): Promise<string | null> => {
  if (_refreshPromise) {
    return _refreshPromise
  }

  _refreshPromise = handleRefresh()

  try {
    const token = await _refreshPromise
    return token
  } finally {
    _refreshPromise = null
  }
}

export const client = async <TData, _TError = unknown, TVariables = unknown>(
  paramsConfig: RequestConfig<TVariables>,
): Promise<ResponseConfig<TData>> => {
  const globalConfig = getConfig()
  const config = { ...globalConfig, ...paramsConfig }

  const normalizedParams = new URLSearchParams()
  Object.entries(config.params || {}).forEach(([key, value]) => {
    if (value !== undefined) {
      normalizedParams.append(key, value === null ? 'null' : value.toString())
    }
  })

  let targetUrl = [config.baseURL, config.url].filter(Boolean).join('')
  if (config.params) {
    targetUrl += `?${normalizedParams}`
  }

  const makeRequest = async () => {
    const headers = new Headers(config.headers)

    if (!headers.has('Content-Type') && config.data) {
      headers.set('Content-Type', 'application/json')
    }

    if (config.accessToken) {
      headers.set('Authorization', `Bearer ${config.accessToken}`)
    }

    return fetch(targetUrl, {
      method: config.method?.toUpperCase(),
      body: JSON.stringify(config.data),
      signal: config.signal,
      headers,
      credentials: 'include',
    })
  }

  let res = await makeRequest()

  if (res.status === 401 && !res.url.endsWith('/auth/token')) {
    const newToken = await getOrHandleRefresh()
    if (newToken) {
      res = await makeRequest()
    }
  }

  const data = [204].includes(res.status) || !res.body ? {} : await res.json()

  if (!res.ok) {
    throw new Error((data as ApiError).message)
  }

  return {
    data: data as TData,
    status: res.status,
    statusText: res.statusText,
    headers: res.headers,
  }
}

client.getConfig = getConfig
client.setConfig = setConfig

export default client
