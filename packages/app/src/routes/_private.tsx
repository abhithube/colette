import { Box, Divider, Flex } from '@colette/ui'
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
  return (
    <Flex h="screen">
      <OuterSidebar />
      <Divider orientation="vertical" />
      <Box w="full" overflowY="auto">
        <Outlet />
      </Box>
    </Flex>
  )
}
