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
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { Link as TLink } from '@tanstack/react-router'
import { History, Home, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { AddBookmarkModal } from './bookmarks/-components/add-bookmark-modal'

export const Route = createFileRoute('/_private/bookmarks')({
  component: Component,
})

function Component() {
  const [isBookmarkModalOpen, setBookmarkModalOpen] = useState(false)

  return (
    <Flex h="full" w="full">
      <Splitter.Root
        gap={0}
        size={[
          {
            id: 'sidebar',
            minSize: 20,
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
                Bookmarks
              </Heading>
              <Dialog.Root
                open={isBookmarkModalOpen}
                onOpenChange={(e) => setBookmarkModalOpen(e.open)}
              >
                <Dialog.Trigger asChild>
                  <IconButton flexShrink={0}>
                    <PlusCircle />
                    New
                  </IconButton>
                </Dialog.Trigger>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                  <AddBookmarkModal close={() => setBookmarkModalOpen(false)} />
                </Dialog.Positioner>
              </Dialog.Root>
            </HStack>
            <VStack alignItems="stretch" px={4} gap={1}>
              <Button asChild variant="ghost">
                <Link asChild textDecoration="none">
                  <TLink
                    to="/bookmarks"
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
                      All Bookmarks
                    </Text>
                  </TLink>
                </Link>
              </Button>
              <Button asChild variant="ghost">
                <Link asChild textDecoration="none">
                  <TLink
                    to="/bookmarks/stash"
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
                      Stash
                    </Text>
                  </TLink>
                </Link>
              </Button>
            </VStack>
            <Divider w="full" />
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
