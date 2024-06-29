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

new Elysia()
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
	.use(auth)
	.use(entries)
	.use(feeds)
	.use(profiles)
	.onError((ctx) => {
		let status = 500
		let message = 'Internal server error'

		if (ctx.error instanceof AppError) {
			message = ctx.error.toString()

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
				message,
			}),
			{
				status,
			},
		)
	})
	.listen(process.env.PORT ?? 3000)
