import { AppSidebar } from './sidebar'
import { getRouteApi, Outlet } from '@colette/router'
import { Sidebar } from '@colette/ui'
import { UserProvider } from '@colette/util'

const routeApi = getRouteApi('/layout')

export const Layout = () => {
  const context = routeApi.useRouteContext()

  return (
    <UserProvider user={context.user}>
      <Sidebar.Provider>
        <>
          <AppSidebar />
          <div className="w-full">
            <Sidebar.Trigger />
            <Outlet />
          </div>
        </>
      </Sidebar.Provider>
    </UserProvider>
  )
}
