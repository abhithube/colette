import { RegisterForm } from './components/register-form'
import { getActiveUserOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { Redirect } from 'wouter'

export const RegisterPage = () => {
  const api = useAPI()

  const query = useQuery({
    ...getActiveUserOptions(api),
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
