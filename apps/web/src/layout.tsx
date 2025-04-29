import { AppSidebar } from './sidebar'
import { Sidebar } from '@colette/ui'
import { Outlet } from '@tanstack/react-router'

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
