import createClient from 'openapi-fetch'
import type { paths } from './openapi'

export const client = createClient<paths>({
	baseUrl: import.meta.env.VITE_BACKEND_URL,
	credentials: 'include',
})

client.use({
	onResponse: async ({ response }) => {
		if (response.status >= 400) {
			const body = await response.json()

			if (response.status === 401) {
				throw new AuthError(body)
			}

			throw new Error(body)
		}
	},
})

export class AuthError extends Error {}
