import Elysia, { t } from 'elysia'
import { ErrorSchema, Nullable } from './common'

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
	.get(
		'/profiles',
		async () => {
			return {
				hasMore: false,
				data: [],
			}
		},
		{
			type: 'application/json',
			response: {
				200: t.Object(
					{
						hasMore: t.Boolean(),
						data: t.Array(t.Ref(ProfileSchema)),
					},
					{
						description: 'Paginated list of profiles',
					},
				),
			},
		},
	)
	.get(
		'/profiles/:id',
		async ({ params: { id } }) => {
			return {
				message: `Profile not found with id: ${id}`,
			}
		},
		{
			params: t.Object({
				id: t.String(),
			}),
			response: {
				200: {
					description: 'Profile by ID',
					...ProfileSchema,
				},
				404: {
					description: 'Profile not found',
					...ErrorSchema,
				},
			},
		},
	)
	.delete(
		'/profiles/:id',
		async ({ params: { id } }) => {
			return {
				message: `Profile not found with id: ${id}`,
			}
		},
		{
			params: t.Object({
				id: t.String(),
			}),
			response: {
				204: t.Null({
					description: 'Profile deleted successfully',
				}),
				404: {
					description: 'Profile not found',
					...ErrorSchema,
				},
			},
		},
	)
