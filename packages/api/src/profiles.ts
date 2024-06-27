import Elysia, { t } from 'elysia'
import { ErrorSchema, Nullable } from './common'
import { profilesService } from './deps'
import session from './plugins/session'

const ProfileSchema = t.Object(
	{
		id: t.String(),
		title: t.String(),
		imageUrl: Nullable(t.String({ format: 'uri' })),
		createdAt: t.String({ format: 'date-time' }),
		updatedAt: t.String({ format: 'date-time' }),
	},
	{
		$id: '#/components/schemas/Profile',
	},
)

export default new Elysia()
	.model({
		Profile: ProfileSchema,
		Error: ErrorSchema,
	})
	.decorate({
		profilesService,
	})
	.use(session)
	.get('/profiles', (ctx) => ctx.profilesService.list(ctx.session), {
		type: 'application/json',
		response: {
			200: t.Object({
				hasMore: t.Boolean(),
				data: t.Array(t.Ref(ProfileSchema)),
			}),
		},
	})
	.get(
		'/profiles/:id',
		(ctx) => ctx.profilesService.get(ctx.params.id, ctx.session),
		{
			params: t.Object({
				id: t.String(),
			}),
			type: 'application/json',
			response: {
				200: 'Profile',
				404: 'Error',
			},
		},
	)
	.delete(
		'/profiles/:id',
		(ctx) => ctx.profilesService.delete(ctx.params.id, ctx.session),
		{
			params: t.Object({
				id: t.String(),
			}),
			type: 'application/json',
			response: {
				204: t.Void({
					description: 'Profile deleted successfully',
				}),
				404: 'Error',
			},
		},
	)
