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
  tokenConfig?: TokenConfig
  oidcConfig?: oidcClient.Configuration
}

type TokenConfig = {
  accessManager: TokenManager
  refreshManager: TokenManager
}

interface TokenManager {
  get: () => string | null
  set: (token: string) => void
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

const refreshToken = async (
  oidcConfig: oidcClient.Configuration,
  tokenConfig: TokenConfig,
): Promise<string | null> => {
  const refreshToken = tokenConfig.refreshManager.get()
  if (!refreshToken) return null

  try {
    const res = await oidcClient.refreshTokenGrant(oidcConfig, refreshToken)

    tokenConfig.accessManager.set(res.access_token)
    if (res.refresh_token) {
      tokenConfig.refreshManager.set(res.refresh_token)
    }

    return res.access_token
  } catch (error) {
    console.error(error)
  }

  return null
}

let _refreshPromise: Promise<string | null> | null = null

const getOrRefreshToken = async (
  oidcConfig: oidcClient.Configuration,
  tokenConfig: TokenConfig,
): Promise<string | null> => {
  if (_refreshPromise) {
    return _refreshPromise
  }

  _refreshPromise = refreshToken(oidcConfig, tokenConfig)

  try {
    const token = await _refreshPromise
    return token
  } finally {
    _refreshPromise = null
  }
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
    const headers = new Headers(config.headers)

    if (!headers.has('Content-Type') && config.data) {
      headers.set('Content-Type', 'application/json')
    }

    if (config.tokenConfig) {
      const accessToken = config.tokenConfig.accessManager.get()
      if (accessToken) {
        headers.set('Authorization', `Bearer ${accessToken}`)
      }
    }

    return fetch(targetUrl, {
      method: config.method?.toUpperCase(),
      body: JSON.stringify(config.data),
      signal: config.signal,
      headers,
    })
  }

  let res = await makeRequest()

  if (res.status === 401 && config.oidcConfig && config.tokenConfig) {
    const newToken = await getOrRefreshToken(
      config.oidcConfig,
      config.tokenConfig,
    )
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
