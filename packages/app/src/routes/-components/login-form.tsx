import { loginOptions } from '@colette/query'
import { Button, Card, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { z } from 'zod'
import { Route } from '../login'

export const LoginForm = () => {
  const context = Route.useRouteContext()

  const navigate = useNavigate()

  const form = useForm({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => login(value),
  })

  const { mutateAsync: login, isPending } = useMutation(
    loginOptions(context.api, context.queryClient, {
      onSuccess: async (user) => {
        form.reset()
        context.user = user

        await navigate({
          to: '/',
          replace: true,
        })
      },
    }),
  )

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Card.Root>
        <Card.Header spaceY="2">
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Body spaceY="4">
          <form.Field
            name="email"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z.string().email('Please enter a valid email'),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <Field.Root
                defaultValue={state.value}
                invalid={state.meta.errors.length > 0}
              >
                <Field.Label>Email</Field.Label>
                <Field.Input
                  placeholder="user@email.com"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <Field.ErrorText>
                  {state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
          <form.Field
            name="password"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z
                .string()
                .min(8, 'Password must be at least 8 characters'),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <Field.Root
                defaultValue={state.value}
                invalid={state.meta.errors.length > 0}
              >
                <Field.Label>Password</Field.Label>
                <Field.Input
                  type="password"
                  placeholder="********"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <Field.ErrorText>
                  {state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </Card.Body>
        <Card.Footer>
          <Button flexGrow={1} loading={isPending}>
            Login
          </Button>
        </Card.Footer>
      </Card.Root>
    </form>
  )
}
