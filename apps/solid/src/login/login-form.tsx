import type { Component } from 'solid-js'
import { Button, Card, Field } from '../lib/components'

export const LoginForm: Component = () => {
  return (
    <form>
      <Card.Root>
        <Card.Header class="space-y-2">
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
          <Field.Root>
            <Field.Label>Email</Field.Label>
            <Field.Input placeholder="user@email.com" />
          </Field.Root>
          <Field.Root>
            <Field.Label>Password</Field.Label>
            <Field.Input type="password" placeholder="********" />
          </Field.Root>
        </Card.Content>
        <Card.Footer>
          <Button class="grow">Login</Button>
        </Card.Footer>
      </Card.Root>
    </form>
  )
}
