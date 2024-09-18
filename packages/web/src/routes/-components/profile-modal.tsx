import {
  Avatar,
  Button,
  Dialog,
  Flex,
  Grid,
  IconButton,
  RadioButtonGroup,
  Text,
} from '@colette/components'
import type { Profile } from '@colette/core'
import { listProfilesOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useQuery } from '@tanstack/react-query'
import { XIcon } from 'lucide-react'
import { Route } from '../_private'

type Props = {
  active: Profile
  close: () => void
}

export function ProfileModal({ active, close }: Props) {
  const context = Route.useRouteContext()

  const { data: profiles } = useQuery(listProfilesOptions(context.api))

  const { Field: TField, handleSubmit } = useForm({
    defaultValues: {
      id: '',
    },
    onSubmit: ({ value }) => {
      if (value.id === active.id) {
        return close()
      }

      console.log(value)
    },
  })

  if (!profiles) return

  return (
    <Dialog.Content p={6}>
      <Dialog.Title>Profile</Dialog.Title>
      <Dialog.Description>Select a profile</Dialog.Description>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          handleSubmit()
        }}
      >
        <TField name="id">
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
        </TField>

        <Flex justify="end" mt={4}>
          <Button>Select</Button>
        </Flex>
      </form>
      <Dialog.CloseTrigger asChild position="absolute" top={2} right={2}>
        <IconButton variant="ghost" size="sm">
          <XIcon />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
