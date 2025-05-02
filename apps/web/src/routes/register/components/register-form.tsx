import { REGISTER_FORM, registerFormOptions } from '@colette/form'
import { useRegisterUserMutation } from '@colette/query'
import { Alert, Button, Card, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi, Link } from '@tanstack/react-router'
import { UserX } from 'lucide-react'

const routeApi = getRouteApi('/register')

export const RegisterForm = () => {
  const context = routeApi.useRouteContext()
  const navigate = routeApi.useNavigate()

  const form = useForm({
    ...registerFormOptions(),
    onSubmit: ({ value, formApi }) =>
      registerUser.mutate(
        {
          email: value.email,
          password: value.password,
        },
        {
          onSuccess: () => {
            formApi.reset()

            navigate({
              to: '/login',
              state: {
                registered: true,
              },
            })
          },
        },
      ),
  })

  const registerUser = useRegisterUserMutation(context.api)

  return (
    <>
      {registerUser.error && (
        <Alert.Root className="mb-4" variant="destructive">
          <UserX />
          <Alert.Title>Failed to log in</Alert.Title>
          <Alert.Description>{registerUser.error.message}</Alert.Description>
        </Alert.Root>
      )}
      <Card.Root>
        <Card.Header>
          <Card.Title>Register</Card.Title>
          <Card.Description>Register a new account</Card.Description>
        </Card.Header>
        <Card.Content>
          <form
            id={REGISTER_FORM}
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault()
              form.handleSubmit()
            }}
          >
            <form.Field name="email">
              {(field) => {
                const errors = field.state.meta.errors

                return (
                  <Field.Root invalid={errors.length !== 0}>
                    <Field.Label>Email</Field.Label>
                    <Field.Input
                      type="email"
                      value={field.state.value}
                      placeholder="user@example.com"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
            <form.Field name="password">
              {(field) => {
                const errors = field.state.meta.errors

                return (
                  <Field.Root invalid={errors.length !== 0}>
                    <Field.Label>Password</Field.Label>
                    <Field.Input
                      type="password"
                      value={field.state.value}
                      placeholder="********"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
            <form.Field
              name="passwordConfirm"
              validators={{
                onBlurListenTo: ['password'],
                onBlur: ({ value, fieldApi }) => {
                  if (value !== fieldApi.form.getFieldValue('password')) {
                    return 'Passwords do not match'
                  }
                },
              }}
            >
              {(field) => {
                const errors = field.state.meta.errors

                return (
                  <Field.Root invalid={errors.length !== 0}>
                    <Field.Label>Confirm Password</Field.Label>
                    <Field.Input
                      type="password"
                      value={field.state.value}
                      placeholder="********"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>
                      {typeof errors[0] === 'string'
                        ? errors[0]
                        : errors[0]?.message}
                    </Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
          </form>
        </Card.Content>
        <Card.Footer className="flex-col items-stretch gap-4">
          <Button form={REGISTER_FORM} disabled={registerUser.isPending}>
            Register
          </Button>
          <div className="self-center text-sm">
            Already have an account?{' '}
            <Link
              className="underline underline-offset-4"
              from="/register"
              to="/login"
            >
              Sign in
            </Link>
          </div>
        </Card.Footer>
      </Card.Root>
    </>
  )
}
