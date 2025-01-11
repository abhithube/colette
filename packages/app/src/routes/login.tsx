import { Box, Center } from '@colette/ui'
import { createFileRoute, redirect } from '@tanstack/react-router'
import { LoginForm } from './-components/login-form'

export const Route = createFileRoute('/login')({
  beforeLoad: ({ context }) => {
    if (context.user) {
      throw redirect({
        to: '/',
      })
    }
  },
  component: Component,
})

function Component() {
  return (
    <Center h="screen">
      <Box w={400}>
        <LoginForm />
      </Box>
    </Center>
  )
}
