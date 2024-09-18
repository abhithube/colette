import {
  Button,
  Dialog,
  Field,
  Flex,
  IconButton,
  VStack,
} from '@colette/components'
import { createBookmarkOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { X } from 'lucide-react'
import { z } from 'zod'
import { Route } from '../../bookmarks'

type Props = {
  close: () => void
}

export function AddBookmarkModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => createBookmark(value),
  })

  const navigate = useNavigate()

  const { mutateAsync: createBookmark, isPending } = useMutation(
    createBookmarkOptions(
      {
        onSuccess: async () => {
          form.reset()
          close()

          await navigate({
            to: '/bookmarks/stash',
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
          form.handleSubmit()
        }}
      >
        <Dialog.Title>Add Bookmark</Dialog.Title>
        <Dialog.Description>Add a bookmark to the stash.</Dialog.Description>
        <VStack alignItems="stretch" spaceY={4} mt={4}>
          <form.Field
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
                <Field.HelperText>URL of the bookmark</Field.HelperText>
                <Field.ErrorText>
                  {state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
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
