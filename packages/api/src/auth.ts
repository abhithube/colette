import Elysia, { t } from 'elysia'
import { ErrorSchema } from './common'
import { authService, lucia } from './deps'
import { ProfileSchema } from './profiles'

const UserSchema = t.Object(
	{
		id: t.String(),
		email: t.String({ format: 'email' }),
		createdAt: t.String({ format: 'date-time' }),
		updatedAt: t.String({ format: 'date-time' }),
	},
	{
		$id: '#/components/schemas/User',
	},
)

const RegisterSchema = t.Object(
	{
		email: t.String({ format: 'email' }),
		password: t.String(),
	},
	{
		$id: '#/components/schemas/Register',
	},
)

const LoginSchema = t.Object(
	{
		email: t.String({ format: 'email' }),
		password: t.String(),
	},
	{
		$id: '#/components/schemas/Login',
	},
)

export default new Elysia()
	.model({
		User: UserSchema,
		Profile: ProfileSchema,
		Register: RegisterSchema,
		Login: LoginSchema,
		Error: ErrorSchema,
	})
	.decorate({
		authService,
	})
	.post('/auth/register', (ctx) => ctx.authService.register(ctx.body), {
		body: 'Register',
		type: 'application/json',
		response: {
			201: 'User',
			400: 'Error',
			409: 'Error',
		},
	})
	.post(
		'/auth/login',
		async (ctx) => {
			const profile = await ctx.authService.login(ctx.body)

			const session = await lucia.createSession(profile.userId, {
				profileId: profile.id,
			})
			const sessionCookie = lucia.createSessionCookie(session.id)
			ctx.cookie[sessionCookie.name].set({
				value: sessionCookie.value,
				...sessionCookie.attributes,
			})

			return profile
		},
		{
			body: 'Login',
			type: 'application/json',
			response: {
				200: 'Profile',
				401: 'Error',
			},
		},
	)
