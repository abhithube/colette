import type { Component } from 'solid-js'
import { LoginForm } from './login-form'

export const LoginPage: Component = () => {
  return (
    <div class="flex h-screen items-center justify-center">
      <div class="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}
