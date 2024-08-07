import { Button } from '@/components/ui/button'
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { UnauthorizedError, UnprocessableContentError } from '@colette/openapi'
import { loginOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../login'

const formSchema = z.object({
	email: z.string().email('Email is not valid.'),
	password: z.string().min(8, 'Password must be 8 or more characters.'),
})

type Values = z.infer<typeof formSchema>

export const LoginForm = () => {
	const context = Route.useRouteContext()

	const [loading, setLoading] = useState(false)

	const navigate = useNavigate()

	const form = useForm<Values>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			email: '',
			password: '',
		},
	})

	const { mutateAsync: login } = useMutation(
		loginOptions(
			{
				onMutate: () => {
					setLoading(true)
				},
				onSuccess: async (profile) => {
					setLoading(false)

					context.profile = profile

					await navigate({
						to: '/',
						replace: true,
					})
				},
				onError: (error) => {
					if (error instanceof UnauthorizedError) {
						form.setError('root', {
							message: error.message,
						})
					} else if (error instanceof UnprocessableContentError) {
						form.setError('root', {
							message: error.message,
						})
					}
				},
			},
			context.api,
		),
	)

	return (
		<Form {...form}>
			<form
				onSubmit={form.handleSubmit((data) => login(data))}
				className="space-y-8"
			>
				<FormField
					control={form.control}
					name="email"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Email</FormLabel>
							<FormControl>
								<Input placeholder="user@email.com" {...field} />
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>
				<FormField
					control={form.control}
					name="password"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Password</FormLabel>
							<FormControl>
								<Input {...field} type="password" />
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>
				<Button disabled={loading}>
					{loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
					Login
				</Button>
			</form>
		</Form>
	)
}
