import {
  Button,
  Dialog,
  Field,
  Fieldset,
  Flex,
  IconButton,
  Switch,
  VStack,
} from '@colette/components'
import { createFeedOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { X } from 'lucide-react'
import { z } from 'zod'
import { Route } from '../../feeds'

type Props = {
  close: () => void
}

export function SubscribeModal({ close }: Props) {
  const context = Route.useRouteContext()

  const {
    Field: TField,
    handleSubmit,
    reset,
  } = useForm({
    defaultValues: {
      url: '',
      pinned: false,
    },
    onSubmit: ({ value }) => createFeed(value),
  })

  const navigate = useNavigate()

  const { mutateAsync: createFeed, isPending } = useMutation(
    createFeedOptions(
      {
        onSuccess: async (data) => {
          reset()
          close()

          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
          })

          await navigate({
            to: '/feeds/$id',
            params: {
              id: data.id,
            },
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Dialog.Content p={6}>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          handleSubmit()
        }}
      >
        <Dialog.Title>Add Feed</Dialog.Title>
        <Dialog.Description>
          Subscribe to a RSS or Atom feed and receive the latest updates.
        </Dialog.Description>
        <VStack alignItems="stretch" spaceY={4} mt={4}>
          <TField
            name="url"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z.string().url('Please enter a valid URL'),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <Field.Root
                defaultValue={state.value}
                invalid={state.meta.errors.length > 0}
              >
                <Field.Label>URL</Field.Label>
                <Field.Input
                  placeholder="https://www.website.com"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <Field.HelperText>URL of the RSS or Atom Feed</Field.HelperText>
                <Field.ErrorText>
                  {state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </TField>
          <TField
            name="pinned"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z.boolean(),
            }}
          >
            {({ handleChange, handleBlur }) => (
              <Fieldset.Root paddingBlock={0} borderTop="none">
                <Fieldset.Legend>Pinned</Fieldset.Legend>
                <Fieldset.HelperText>
                  Should the feed be pinned to the sidebar?
                </Fieldset.HelperText>
                <Field.Root>
                  <Switch
                    onCheckedChange={(details) => handleChange(details.checked)}
                    onBlur={handleBlur}
                  />
                </Field.Root>
              </Fieldset.Root>
            )}
          </TField>
          <Flex justify="end">
            <Button loading={isPending}>Submit</Button>
          </Flex>
        </VStack>
      </form>
      <Dialog.CloseTrigger asChild position="absolute" top="2" right="2">
        <IconButton variant="ghost" size="sm">
          <X />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
