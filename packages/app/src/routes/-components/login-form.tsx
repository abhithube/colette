import { loginOptions } from '@colette/query'
import { FormMessage } from '@colette/react-ui/components/form'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@colette/react-ui/components/ui/card'
import { Input } from '@colette/react-ui/components/ui/input'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
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
      <Card>
        <CardHeader>
          <CardTitle>Login</CardTitle>
          <CardDescription>Login to your account</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <form.Field
            name="email"
            validators={{
              onBlur: z.string().email('Please enter a valid email'),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <div className="space-y-1">
                <Label>Email</Label>
                <Input
                  placeholder="user@example.com"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
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
            {({ state, handleChange, handleBlur }) => (
              <div className="space-y-1">
                <Label>Password</Label>
                <Input
                  type="password"
                  placeholder="********"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
            )}
          </form.Field>
        </CardContent>
        <CardFooter>
          <Button className="flex-1" disabled={isPending}>
            Login
          </Button>
        </CardFooter>
      </Card>
    </form>
  )
}
