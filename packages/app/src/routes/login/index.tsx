import type { FC } from 'react'
import { LoginForm } from './components/login-form'

export const LoginPage: FC = () => {
  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}
