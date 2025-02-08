import { LoginForm } from './components/login-form'
import type { FC } from 'react'

export const LoginPage: FC = () => {
  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}
