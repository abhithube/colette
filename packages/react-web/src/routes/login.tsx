import { createFileRoute, redirect } from '@tanstack/react-router'
import { LoginForm } from './-components/login-form'

export const Route = createFileRoute('/login')({
	beforeLoad: ({ context }) => {
		if (context.profile) {
			throw redirect({
				to: '/',
			})
		}
	},
	component: LoginForm,
})
