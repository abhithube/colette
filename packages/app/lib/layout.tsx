import { AppSidebar } from './sidebar'
import { Outlet } from '@colette/router'
import { Sidebar } from '@colette/ui'

export const Layout = () => {
  return (
    <Sidebar.Provider>
      <>
        <AppSidebar />
        <div className="w-full">
          <Sidebar.Trigger />
          <Outlet />
        </div>
      </>
    </Sidebar.Provider>
  )
}
