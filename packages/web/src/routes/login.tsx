import { createFileRoute, redirect } from '@tanstack/react-router'
import { LoginForm } from './-components/login-form'

export const Route = createFileRoute('/login')({
  beforeLoad: ({ context }) => {
    if (context.profile) {
      throw redirect({
        to: '/',
      })
    }
  },
  component: Component,
})

function Component() {
  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}
