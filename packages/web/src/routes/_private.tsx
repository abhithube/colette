import { Box, Divider, Flex } from '@colette/components'
import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { OuterSidebar } from './-components/outer-sidebar'

export const Route = createFileRoute('/_private')({
  beforeLoad: async ({ context }) => {
    if (!context.profile) {
      throw redirect({
        to: '/login',
      })
    }

    return {
      profile: context.profile,
    }
  },
  component: Component,
})

function Component() {
  const { profile } = Route.useRouteContext()

  if (!profile) return

  return (
    <Flex h="screen">
      <OuterSidebar profile={profile} />
      <Divider orientation="vertical" />
      <Box w="full" overflowY="auto">
        <Outlet />
      </Box>
    </Flex>
  )
}
