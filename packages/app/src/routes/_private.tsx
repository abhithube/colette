import { Box, Divider, Flex } from '@colette/ui'
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
    <Flex h="screen">
      <OuterSidebar />
      <Divider orientation="vertical" />
      <Box w="full" overflowY="auto">
        <Outlet />
      </Box>
    </Flex>
  )
}
