import { loginOptions } from '@colette/solid-query'
import { createForm } from '@tanstack/solid-form'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import type { Component } from 'solid-js'
import { z } from 'zod'
import { useAPI } from '../lib/api-context'
import { Button } from '../lib/components/button'
import {
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardRoot,
  CardTitle,
} from '../lib/components/card'
import { FormControl, FormMessage } from '../lib/components/form'
import { Input } from '../lib/components/input'
import { Label } from '../lib/components/label'

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
      <CardRoot>
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
              <FormControl invalid={field().state.meta.errors.length > 0}>
                <Label>Email</Label>
                <Input
                  placeholder="user@email.com"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <FormMessage>
                  {field().state.meta.errors[0]?.toString()}
                </FormMessage>
              </FormControl>
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
              <FormControl invalid={field().state.meta.errors.length > 0}>
                <Label>Password</Label>
                <Input
                  type="password"
                  placeholder="********"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <FormMessage>
                  {field().state.meta.errors[0]?.toString()}
                </FormMessage>
              </FormControl>
            )}
          </form.Field>
        </CardContent>
        <CardFooter>
          <Button class="grow" disabled={isPending}>
            Login
          </Button>
        </CardFooter>
      </CardRoot>
    </form>
  )
}
