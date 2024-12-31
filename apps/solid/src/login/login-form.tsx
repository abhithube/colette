import { loginOptions } from '@colette/solid-query'
import { createForm } from '@tanstack/solid-form'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import type { Component } from 'solid-js'
import { z } from 'zod'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '~/components/ui/text-field'
import { useAPI } from '~/lib/api-context'

export const LoginForm: Component = () => {
  const form = createForm(() => ({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => login(value),
  }))

  const { mutateAsync: login, isPending } = createMutation(() =>
    loginOptions(
      {
        onSuccess: async (user) => {
          console.log(user)

          form.reset()
        },
      },
      useAPI(),
      useQueryClient(),
    ),
  )

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Card>
        <CardHeader class="space-y-2">
          <CardTitle>Login</CardTitle>
          <CardDescription>Login to your account</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <form.Field
            name="email"
            validators={{
              onBlur: z.string().email('Email is not valid'),
            }}
          >
            {(field) => (
              <TextField
                validationState={
                  field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
                }
              >
                <TextFieldLabel>Email</TextFieldLabel>
                <TextFieldInput
                  placeholder="user@email.com"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <TextFieldErrorMessage>
                  {field().state.meta.errors[0]?.toString()}
                </TextFieldErrorMessage>
              </TextField>
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
              <TextField
                validationState={
                  field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
                }
              >
                <TextFieldLabel>Password</TextFieldLabel>
                <TextFieldInput
                  type="password"
                  placeholder="********"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <TextFieldErrorMessage>
                  {field().state.meta.errors[0]?.toString()}
                </TextFieldErrorMessage>
              </TextField>
            )}
          </form.Field>
        </CardContent>
        <CardFooter>
          <Button class="grow" disabled={isPending}>
            Login
          </Button>
        </CardFooter>
      </Card>
    </form>
  )
}
