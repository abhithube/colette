import { useLoginUserMutation } from '@colette/query'
import { Alert, Card, Button, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { UserCheck } from 'lucide-react'
import { Link, useSearchParams } from 'wouter'
import { navigate, useHistoryState } from 'wouter/use-browser-location'
import { z } from 'zod'

export const LoginForm = () => {
  const [searchParams] = useSearchParams()
  const history = useHistoryState<{ registered?: boolean }>()

  const form = useForm({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) =>
      login.mutate(value, {
        onSuccess: () => {
          form.reset()

          const redirect = searchParams.get('redirect')
          if (redirect) {
            navigate(redirect, {
              replace: true,
            })
          }
        },
      }),
  })

  const login = useLoginUserMutation()

  return (
    <>
      {history?.registered && (
        <Alert.Root className="mb-4">
          <UserCheck />
          <Alert.Title>Registered!</Alert.Title>
          <Alert.Description>Your account has been created.</Alert.Description>
        </Alert.Root>
      )}
      <Card.Root>
        <Card.Header>
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Content>
          <form
            id="login"
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault()
              form.handleSubmit()
            }}
          >
            <form.Field
              name="email"
              validators={{
                onBlur: z.string().email('Please enter a valid email'),
              }}
            >
              {(field) => (
                <Field.Root className="space-y-2">
                  <Field.Label>Email</Field.Label>
                  <Field.Input
                    type="email"
                    value={field.state.value}
                    placeholder="user@example.com"
                    onChange={(e) => field.handleChange(e.target.value)}
                    onBlur={field.handleBlur}
                  />
                  <Field.ErrorText>
                    {field.state.meta.errors[0]?.toString()}
                  </Field.ErrorText>
                </Field.Root>
              )}
            </form.Field>
            <form.Field
              name="password"
              validators={{
                onBlur: z
                  .string()
                  .min(8, 'Password must be at least 8 characters'),
              }}
            >
              {(field) => (
                <Field.Root className="space-y-2">
                  <Field.Label>Password</Field.Label>
                  <Field.Input
                    type="password"
                    value={field.state.value}
                    placeholder="********"
                    onChange={(e) => field.handleChange(e.target.value)}
                    onBlur={field.handleBlur}
                  />
                  <Field.ErrorText>
                    {field.state.meta.errors[0]?.toString()}
                  </Field.ErrorText>
                </Field.Root>
              )}
            </form.Field>
          </form>
        </Card.Content>
        <Card.Footer className="flex-col items-stretch gap-4">
          <Button form="login" disabled={login.isPending}>
            Login
          </Button>
          <div className="self-center text-sm">
            {"Don't have an account? "}
            <Link className="underline underline-offset-4" to="/register">
              Sign up
            </Link>
          </div>
        </Card.Footer>
      </Card.Root>
    </>
  )
}
