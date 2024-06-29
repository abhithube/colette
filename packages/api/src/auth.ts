import Elysia, { t } from 'elysia'
import { ErrorSchema } from './common'
import { authService, lucia } from './deps'
import session from './plugins/session'
import { ProfileSchema } from './profiles'

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
		Profile: ProfileSchema,
		Register: RegisterSchema,
		Login: LoginSchema,
		Error: ErrorSchema,
	})
	.decorate({
		authService,
	})
	.use(session)
	.post('/auth/register', (ctx) => ctx.authService.register(ctx.body), {
		body: 'Register',
		type: 'application/json',
		response: {
			201: 'Profile',
			400: 'Error',
			409: 'Error',
		},
	})
	.post(
		'/auth/login',
		async (ctx) => {
			const profile = await ctx.authService.login(ctx.body)

			const session = await lucia.createSession(profile.userId, {
				profile_id: profile.id,
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
