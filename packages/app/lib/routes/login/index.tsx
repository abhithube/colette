import { LoginForm } from './components/login-form'
import { getRouteApi } from '@colette/router'

const routeApi = getRouteApi('/login')

export const LoginPage = () => {
  const { loggedOut } = routeApi.useSearch()

  return (
    <div className="flex h-screen items-center justify-center">
      <div className="w-[400px]">
        <LoginForm loggedOut={loggedOut} />
      </div>
    </div>
  )
}
