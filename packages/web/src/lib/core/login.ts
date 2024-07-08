import * as v from 'valibot'
import { APIError, client } from '../client'

export const loginSchema = v.object({
	email: v.pipe(
		v.string(),
		v.nonEmpty('Please enter your email.'),
		v.email('The email provided is not valid.'),
	),
	password: v.pipe(
		v.string(),
		v.nonEmpty('Please enter a password.'),
		v.minLength(8, 'Your password must have at least 8 characters.'),
	),
})

export type LoginDTO = v.InferInput<typeof loginSchema>

class AuthAPI {
	async login(body: LoginDTO) {
		const res = await client.POST('/api/v1/auth/login', {
			body,
		})
		if (res.error) {
			throw new APIError(res.error.message)
		}

		return res.data
	}
}

export const authAPI = new AuthAPI()
