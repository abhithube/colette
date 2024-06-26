import swagger from '@elysiajs/swagger'
import Elysia from 'elysia'
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
	.use(entries)
	.use(feeds)
	.use(profiles)
	.listen(process.env.PORT ?? 3000)
