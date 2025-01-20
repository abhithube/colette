import { Separator } from '@colette/react-ui/components/ui/separator'
import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { OuterSidebar } from './-components/outer-sidebar'

export const Route = createFileRoute('/_private')({
  beforeLoad: async ({ context }) => {
    if (!context.user) {
      throw redirect({
        to: '/login',
      })
    }

    return {
      user: context.user,
    }
  },
  component: Component,
})

function Component() {
  return (
    <div className="flex h-screen">
      <OuterSidebar />
      <Separator orientation="vertical" />
      <div className="w-full overflow-y-auto">
        <Outlet />
      </div>
    </div>
  )
}
