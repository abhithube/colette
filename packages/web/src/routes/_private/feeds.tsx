import {
  Box,
  Button,
  Dialog,
  Divider,
  Flex,
  HStack,
  Heading,
  Icon,
  IconButton,
  Link,
  Splitter,
  Text,
  VStack,
  css,
} from '@colette/components'
import { listFeedsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { Outlet, Link as TLink, createFileRoute } from '@tanstack/react-router'
import { History, Home, PlusCircle, Wrench } from 'lucide-react'
import { FeedItem } from './feeds/-components/feed-item'
import { SubscribeModal } from './feeds/-components/subscribe-modal'

export const Route = createFileRoute('/_private/feeds')({
  loader: async ({ context }) => {
    const options = listFeedsOptions(
      { pinned: true, filterByTags: true, 'tag[]': [] },
      context.profile.id,
      context.api,
    )

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

  if (!feeds) return

  return (
    <Flex h="full" w="full">
      <Splitter.Root
        gap={0}
        size={[
          {
            id: 'sidebar',
            minSize: 15,
            size: 20,
            maxSize: 30,
          },
          {
            id: 'main',
          },
        ]}
      >
        <Splitter.Panel id="sidebar" border="none">
          <Box h="full" w="full" py={4} spaceY={4} overflowY="auto">
            <HStack justify="space-between" px={4}>
              <Heading as="h2" fontSize="3xl" fontWeight="medium">
                Feeds
              </Heading>
              <Dialog.Root>
                <Dialog.Trigger asChild>
                  <IconButton variant="outline" flexShrink={0}>
                    <PlusCircle />
                    New
                  </IconButton>
                </Dialog.Trigger>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                  <Dialog.Context>
                    {({ setOpen }) => (
                      <SubscribeModal close={() => setOpen(false)} />
                    )}
                  </Dialog.Context>
                </Dialog.Positioner>
              </Dialog.Root>
            </HStack>
            <VStack alignItems="stretch" px={4} gap={1}>
              <Button asChild variant="ghost">
                <Link asChild textDecoration="none">
                  <TLink
                    to="/feeds"
                    activeProps={{
                      className: css({
                        bg: 'bg.muted',
                      }),
                    }}
                    activeOptions={{
                      exact: true,
                    }}
                  >
                    <Icon>
                      <Home />
                    </Icon>
                    <Text as="span" flexGrow={1} truncate>
                      All Feeds
                    </Text>
                  </TLink>
                </Link>
              </Button>
              <Button asChild variant="ghost">
                <Link asChild textDecoration="none">
                  <TLink
                    to="/feeds/archived"
                    activeProps={{
                      className: css({
                        bg: 'bg.muted',
                      }),
                    }}
                  >
                    <Icon>
                      <History />
                    </Icon>
                    <Text as="span" flexGrow={1} truncate>
                      Archived
                    </Text>
                  </TLink>
                </Link>
              </Button>
              <Button asChild variant="ghost">
                <Link asChild textDecoration="none">
                  <TLink
                    to="/feeds/manage"
                    activeProps={{
                      className: css({
                        bg: 'bg.muted',
                      }),
                    }}
                  >
                    <Icon>
                      <Wrench />
                    </Icon>
                    <Text as="span" flexGrow={1} truncate>
                      Manage Feeds
                    </Text>
                  </TLink>
                </Link>
              </Button>
            </VStack>
            {feeds.data.length > 0 && (
              <>
                <Divider w="full" />
                <Box>
                  <Flex
                    justify="space-between"
                    alignItems="center"
                    mb={2}
                    px={4}
                  >
                    <Text
                      as="span"
                      fontSize="xs"
                      fontWeight="semibold"
                      color="fg.muted"
                      flexGrow={1}
                    >
                      Pinned
                    </Text>
                  </Flex>
                  <Box mt={1} h="full" spaceY={1} px={4} overflowY="auto">
                    {feeds.data.map((feed) => (
                      <FeedItem key={feed.id} feed={feed} />
                    ))}
                  </Box>
                </Box>
              </>
            )}
          </Box>
        </Splitter.Panel>
        <Splitter.ResizeTrigger
          id="sidebar:main"
          m={0}
          rounded="none"
          borderInlineEndWidth="1px"
          minW={0}
        />
        <Splitter.Panel id="main" border="none">
          <Box w="full" h="screen" overflowY="auto">
            <Outlet />
          </Box>
        </Splitter.Panel>
      </Splitter.Root>
    </Flex>
  )
}
