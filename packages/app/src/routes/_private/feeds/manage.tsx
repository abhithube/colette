import { listFeedsOptions } from '@colette/query'
import { HStack, Heading } from '@colette/ui'
import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedGrid } from './-components/feed-grid'

export const Route = createFileRoute('/_private/feeds/manage')({
  loader: async ({ context }) => {
    const options = listFeedsOptions({}, context.profile.id, context.api)

    await context.queryClient.ensureQueryData(options)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const { data: feeds } = useQuery(options)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!feeds) return

  return (
    <>
      <HStack pos="sticky" zIndex="sticky" top={0} bg="bg.default" p={8}>
        <Heading as="h1" fontSize="3xl" fontWeight="medium">
          Manage Feeds
        </Heading>
      </HStack>
      <main>
        <FeedGrid feeds={feeds.data} />
      </main>
    </>
  )
}
