import { listProfilesOptions, switchProfileOptions } from '@colette/query'
import {
  Avatar,
  Button,
  Dialog,
  Flex,
  Grid,
  IconButton,
  RadioButtonGroup,
  Text,
} from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQuery } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { XIcon } from 'lucide-react'
import { useState } from 'react'
import { Route } from '../_private'
import { AddProfileForm } from './add-profile-form'

type Props = {
  close: () => void
}

export function ProfileModal({ close }: Props) {
  const context = Route.useRouteContext()

  const navigate = useNavigate()

  const { data: profiles } = useQuery(listProfilesOptions(context.api))

  const form = useForm({
    defaultValues: {
      id: '',
    },
    onSubmit: ({ value }) => {
      if (value.id === context.profile.id) {
        return close()
      }

      return login(value)
    },
  })

  const { mutateAsync: login, isPending } = useMutation(
    switchProfileOptions(
      {
        onSuccess: async (profile) => {
          close()

          context.profile = profile

          await navigate({
            to: '/',
            replace: true,
          })
        },
      },
      context.api,
    ),
  )

  const [creatingProfile, setCreatingProfile] = useState(false)

  if (!profiles) return

  return (
    <Dialog.Content p={6}>
      {creatingProfile ? (
        <AddProfileForm back={() => setCreatingProfile(false)} />
      ) : (
        <>
          <Dialog.Title>Profile</Dialog.Title>
          <Dialog.Description>Select a profile</Dialog.Description>
          <form
            onSubmit={(e) => {
              e.preventDefault()
              form.handleSubmit()
            }}
          >
            <form.Field name="id">
              {({ handleChange }) => (
                <RadioButtonGroup.Root
                  asChild
                  variant="outline"
                  mt={4}
                  onValueChange={(details) => handleChange(details.value)}
                >
                  <Grid columns={{ base: 1, md: 2, lg: 3 }} gap={4}>
                    {profiles.data.map((profile) => (
                      <RadioButtonGroup.Item
                        key={profile.id}
                        value={profile.id}
                        h="unset"
                        p={4}
                      >
                        <RadioButtonGroup.ItemControl />
                        <RadioButtonGroup.ItemText spaceX={2}>
                          <Avatar
                            src={profile.imageUrl ?? undefined}
                            name={profile.title[0]}
                          />
                          <Text as="span">{profile.title}</Text>
                        </RadioButtonGroup.ItemText>
                        <RadioButtonGroup.ItemHiddenInput />
                      </RadioButtonGroup.Item>
                    ))}
                  </Grid>
                </RadioButtonGroup.Root>
              )}
            </form.Field>

            <Flex justify="end" gap={2} mt={4}>
              <Button
                variant="outline"
                onClick={() => setCreatingProfile(true)}
              >
                Create new
              </Button>
              <Button loading={isPending} type="submit">
                Select
              </Button>
            </Flex>
          </form>
          <Dialog.CloseTrigger asChild position="absolute" top={2} right={2}>
            <IconButton variant="ghost" size="sm">
              <XIcon />
            </IconButton>
          </Dialog.CloseTrigger>
        </>
      )}
    </Dialog.Content>
  )
}
