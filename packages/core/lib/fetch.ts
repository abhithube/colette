import * as oidcClient from 'openid-client'

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
  headers?: HeadersInit
  oidcConfig?: oidcClient.Configuration
}

export type ResponseConfig<TData = unknown> = {
  data: TData
  status: number
  statusText: string
  headers: Headers
}

let _config: Partial<RequestConfig> = {}

const getConfig = () => _config

const setConfig = (config: Partial<RequestConfig>) => {
  _config = config
  return getConfig()
}

export type ResponseErrorConfig<TError = unknown> = TError

const refreshToken = async (
  oidcConfig: oidcClient.Configuration,
): Promise<string | null> => {
  const refreshToken = localStorage.getItem('colette-refresh-token')
  if (!refreshToken) return null

  try {
    const res = await oidcClient.refreshTokenGrant(oidcConfig, refreshToken, {})

    localStorage.setItem('colette-access-token', res.access_token)
    if (res.refresh_token) {
      localStorage.setItem('colette-refresh-token', res.refresh_token)
    }

    return res.access_token
  } catch (error) {
    console.error(error)
  }

  return null
}

export const client = async <TData, TError = unknown, TVariables = unknown>(
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
    const token = localStorage.getItem('colette-access-token')

    const headers = new Headers(config.headers)
    if (token) {
      headers.set('Authorization', `Bearer ${token}`)
    }

    return fetch(targetUrl, {
      method: config.method?.toUpperCase(),
      body: JSON.stringify(config.data),
      signal: config.signal,
      headers,
    })
  }

  let res = await makeRequest()

  if (res.status === 401 && config.oidcConfig) {
    const newToken = await refreshToken(config.oidcConfig)
    if (newToken) {
      res = await makeRequest()
    }
  }

  const data =
    [204, 205, 304].includes(res.status) || !res.body ? {} : await res.json()

  return {
    data,
    status: res.status,
    statusText: res.statusText,
    headers: res.headers,
  }
}

client.getConfig = getConfig
client.setConfig = setConfig

export default client
