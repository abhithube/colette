import { useLoginMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { UserCheck } from 'lucide-react'
import type { FC } from 'react'
import { Link, useSearchParams } from 'wouter'
import { navigate, useHistoryState } from 'wouter/use-browser-location'
import { z } from 'zod'
import { FormMessage } from '~/components/form'
import { Alert, AlertDescription, AlertTitle } from '~/components/ui/alert'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const LoginForm: FC = () => {
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

  const login = useLoginMutation()

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      {history?.registered && (
        <Alert>
          <UserCheck />
          <AlertTitle>Registered!</AlertTitle>
          <AlertDescription>Your account has been created.</AlertDescription>
        </Alert>
      )}
      <Card className="mt-4">
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
            {(field) => (
              <div className="space-y-1">
                <Label>Email</Label>
                <Input
                  type="email"
                  value={field.state.value}
                  placeholder="user@example.com"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
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
            {(field) => (
              <div className="space-y-1">
                <Label>Password</Label>
                <Input
                  type="password"
                  value={field.state.value}
                  placeholder="********"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
        </CardContent>
        <CardFooter className="flex-col items-stretch gap-4">
          <Button disabled={login.isPending}>Login</Button>
          <div className="self-center text-sm">
            {"Don't have an account? "}
            <Link className="underline underline-offset-4" to="/register">
              Sign up
            </Link>
          </div>
        </CardFooter>
      </Card>
    </form>
  )
}
