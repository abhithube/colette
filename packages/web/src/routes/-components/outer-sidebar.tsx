import {
  Dialog,
  Flex,
  IconButton,
  Tooltip,
  VStack,
  css,
} from '@colette/components'
import type { Profile } from '@colette/core'
import { Link } from '@tanstack/react-router'
import { Bookmark, Home, Rss, Search, Settings, User } from 'lucide-react'
import { useState } from 'react'
import { ProfileModal } from './profile-modal'
import { SettingsModal } from './settings-modal'

type Props = {
  profile: Profile
}

export const OuterSidebar = ({ profile }: Props) => {
  const [isProfileModalOpen, setProfileModalOpen] = useState(false)
  const [isSettingsModalOpen, setSettingsModalOpen] = useState(false)

  return (
    <VStack h="full" p={4}>
      <Tooltip.Root
        positioning={{
          placement: 'right-start',
        }}
      >
        <Tooltip.Trigger asChild>
          <IconButton asChild variant="ghost" size="lg">
            <Link
              to="/"
              activeProps={{
                className: css({
                  bg: 'bg.muted',
                }),
              }}
            >
              <Home />
            </Link>
          </IconButton>
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
          <IconButton asChild variant="ghost" size="lg">
            <Link
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
              <Rss />
            </Link>
          </IconButton>
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
          <IconButton asChild variant="ghost" size="lg">
            <Link
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
              <Bookmark />
            </Link>
          </IconButton>
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
      <Dialog.Root
        open={isProfileModalOpen}
        onOpenChange={(e) => setProfileModalOpen(e.open)}
      >
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
          <ProfileModal
            profile={profile}
            close={() => setProfileModalOpen(false)}
          />
        </Dialog.Positioner>
      </Dialog.Root>
      <Dialog.Root
        open={isSettingsModalOpen}
        onOpenChange={(e) => setSettingsModalOpen(e.open)}
      >
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
          <SettingsModal close={() => setSettingsModalOpen(false)} />
        </Dialog.Positioner>
      </Dialog.Root>
    </VStack>
  )
}
