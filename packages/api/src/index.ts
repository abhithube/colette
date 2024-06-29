import {
	AppError,
	ConflictError,
	NotFoundError,
	UserNotAuthenticatedError,
} from '@colette/core'
import swagger from '@elysiajs/swagger'
import Elysia from 'elysia'
import auth from './auth'
import entries from './entries'
import feeds from './feeds'
import profiles from './profiles'

new Elysia({ aot: false })
	.use(
		swagger({
			documentation: {
				info: {
					title: 'Colette API',
					version: '1.0.0',
					description: 'REST API for Colette feeds/bookmarks aggregator',
				},
			},
		}),
	)
	.onError((ctx) => {
		let status = 500
		let name = 'Error'
		let message = 'Internal server error'

		console.log(ctx.error)

		if (ctx.error instanceof AppError) {
			name = ctx.error.name
			message = ctx.error.message

			if (ctx.error instanceof UserNotAuthenticatedError) {
				status = 401
			} else if (ctx.error instanceof NotFoundError) {
				status = 404
			} else if (ctx.error instanceof ConflictError) {
				status = 409
			} else {
				status = 500
			}
		}

		return new Response(
			JSON.stringify({
				name,
				message,
			}),
			{
				status,
			},
		)
	})
	.use(auth)
	.use(entries)
	.use(feeds)
	.use(profiles)
	.listen(process.env.PORT ?? 3000)
