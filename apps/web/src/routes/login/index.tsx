import { LoginForm } from './components/login-form'
import { getActiveOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import type { FC } from 'react'
import { Redirect } from 'wouter'

export const LoginPage: FC = () => {
  const api = useAPI()

  const query = useQuery({
    ...getActiveOptions(api),
    retry: false,
  })

  if (query.isLoading) return

  if (query.data) {
    return <Redirect to="/" replace />
  }

  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}
