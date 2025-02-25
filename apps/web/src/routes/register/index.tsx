import { RegisterForm } from './components/register-form'
import { getActiveOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import type { FC } from 'react'
import { Redirect } from 'wouter'

export const RegisterPage: FC = () => {
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
        <RegisterForm />
      </div>
    </div>
  )
}
