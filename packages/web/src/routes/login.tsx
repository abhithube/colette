import { type SubmitHandler, createForm, valiForm } from '@modular-forms/solid'
import {
	type Navigator,
	action,
	useAction,
	useNavigate,
	useSubmission,
} from '@solidjs/router'
import { type Component, Show } from 'solid-js'
import { Button } from '~/components/ui/button'
import { FormLabel } from '~/components/ui/form-label'
import { Input } from '~/components/ui/input'
import { APIError } from '../lib/client'
import { type LoginDTO, authAPI, loginSchema } from '../lib/core/login'

const loginAction = action(async (data: LoginDTO, navigate: Navigator) => {
	try {
		const profile = await authAPI.login(data)

		navigate('/', {
			replace: true,
			state: profile,
		})
	} catch (error) {
		if (error instanceof APIError) {
			return error.message
		}

		throw new Error()
	}
})

export const Login: Component = () => {
	const navigate = useNavigate()

	const login = useAction(loginAction)
	const submission = useSubmission(loginAction)

	const [loginForm, { Form, Field }] = createForm<LoginDTO>({
		validate: valiForm(loginSchema),
	})

	const handleSubmit: SubmitHandler<LoginDTO> = async (values) => {
		await login(values, navigate)
	}

	return (
		<div>
			<h1>Login</h1>
			<Show when={submission.result}>
				<div>{submission.result}</div>
			</Show>
			<Form onSubmit={handleSubmit}>
				<Field name="email">
					{(field, props) => (
						<div>
							<FormLabel for={props.name}>Email</FormLabel>
							<Input
								{...props}
								id={props.name}
								type="email"
								placeholder="Enter your email"
								required
							/>
							{field.error && <div>{field.error}</div>}
						</div>
					)}
				</Field>
				<Field name="password">
					{(field, props) => (
						<div>
							<FormLabel for={props.name}>Password</FormLabel>
							<Input
								{...props}
								id={props.name}
								type="password"
								placeholder="Enter your password"
								required
							/>
							{field.error && <div>{field.error}</div>}
						</div>
					)}
				</Field>
				<Button type="submit" disabled={loginForm.submitting}>
					Login
				</Button>
			</Form>
		</div>
	)
}
