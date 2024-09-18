import {
  Button,
  Dialog,
  Flex,
  Icon,
  IconButton,
  Link,
  Tooltip,
  VStack,
  css,
} from '@colette/components'
import { Link as TLink } from '@tanstack/react-router'
import { Bookmark, Home, Rss, Search, Settings, User } from 'lucide-react'
import { ProfileModal } from './profile-modal'
import { SettingsModal } from './settings-modal'

export const OuterSidebar = () => {
  return (
    <VStack h="full" p={4}>
      <Tooltip.Root
        positioning={{
          placement: 'right-start',
        }}
      >
        <Tooltip.Trigger asChild>
          <Button asChild variant="ghost" size="lg">
            <Link asChild>
              <TLink
                to="/"
                activeProps={{
                  className: css({
                    bg: 'bg.muted',
                  }),
                }}
              >
                <Icon>
                  <Home />
                </Icon>
              </TLink>
            </Link>
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Positioner>
          <Tooltip.Arrow>
            <Tooltip.ArrowTip />
          </Tooltip.Arrow>
          <Tooltip.Content>Home</Tooltip.Content>
        </Tooltip.Positioner>
      </Tooltip.Root>
      <Tooltip.Root
        positioning={{
          placement: 'right',
        }}
      >
        <Tooltip.Trigger asChild>
          <Button asChild variant="ghost" size="lg">
            <Link asChild>
              <TLink
                to="/feeds"
                activeProps={{
                  className: css({
                    bg: 'bg.muted',
                  }),
                }}
                activeOptions={{
                  exact: false,
                }}
              >
                <Icon>
                  <Rss />
                </Icon>
              </TLink>
            </Link>
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Positioner>
          <Tooltip.Arrow>
            <Tooltip.ArrowTip />
          </Tooltip.Arrow>
          <Tooltip.Content>Feed</Tooltip.Content>
        </Tooltip.Positioner>
      </Tooltip.Root>
      <Tooltip.Root
        positioning={{
          placement: 'right',
        }}
      >
        <Tooltip.Trigger asChild>
          <Button asChild variant="ghost" size="lg">
            <Link asChild>
              <TLink
                to="/bookmarks"
                activeProps={{
                  className: css({
                    bg: 'bg.muted',
                  }),
                }}
                activeOptions={{
                  exact: false,
                }}
              >
                <Icon>
                  <Bookmark />
                </Icon>
              </TLink>
            </Link>
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Positioner>
          <Tooltip.Arrow>
            <Tooltip.ArrowTip />
          </Tooltip.Arrow>
          <Tooltip.Content>Bookmarks</Tooltip.Content>
        </Tooltip.Positioner>
      </Tooltip.Root>
      <Tooltip.Root
        positioning={{
          placement: 'right',
        }}
      >
        <Tooltip.Trigger asChild>
          <IconButton variant="ghost" size="lg">
            <Search />
          </IconButton>
        </Tooltip.Trigger>
        <Tooltip.Positioner>
          <Tooltip.Arrow>
            <Tooltip.ArrowTip />
          </Tooltip.Arrow>
          <Tooltip.Content>Search</Tooltip.Content>
        </Tooltip.Positioner>
      </Tooltip.Root>
      <Flex grow={1} />
      <Dialog.Root>
        <Tooltip.Root
          positioning={{
            placement: 'right',
          }}
        >
          <Tooltip.Trigger asChild>
            <Dialog.Trigger asChild>
              <IconButton variant="ghost" size="lg">
                <User />
              </IconButton>
            </Dialog.Trigger>
          </Tooltip.Trigger>
          <Tooltip.Positioner>
            <Tooltip.Arrow>
              <Tooltip.ArrowTip />
            </Tooltip.Arrow>
            <Tooltip.Content>Profile</Tooltip.Content>
          </Tooltip.Positioner>
        </Tooltip.Root>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Context>
            {({ setOpen }) => <ProfileModal close={() => setOpen(false)} />}
          </Dialog.Context>
        </Dialog.Positioner>
      </Dialog.Root>
      <Dialog.Root>
        <Tooltip.Root
          positioning={{
            placement: 'right',
          }}
        >
          <Tooltip.Trigger asChild>
            <Dialog.Trigger asChild>
              <IconButton variant="ghost" size="lg">
                <Settings />
              </IconButton>
            </Dialog.Trigger>
          </Tooltip.Trigger>
          <Tooltip.Positioner>
            <Tooltip.Arrow>
              <Tooltip.ArrowTip />
            </Tooltip.Arrow>
            <Tooltip.Content>Settings</Tooltip.Content>
          </Tooltip.Positioner>
        </Tooltip.Root>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Context>
            {({ setOpen }) => <SettingsModal close={() => setOpen(false)} />}
          </Dialog.Context>
        </Dialog.Positioner>
      </Dialog.Root>
    </VStack>
  )
}
