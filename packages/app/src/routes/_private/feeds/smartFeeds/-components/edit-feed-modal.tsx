import type { SmartFeed } from '@colette/core'
import { updateSmartFeedOptions } from '@colette/query'
import { Button, Dialog, Field, Flex, IconButton, VStack } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { X } from 'lucide-react'
import { useEffect } from 'react'
import { z } from 'zod'
import { Route } from '../$id'

type Props = {
  smartFeed: SmartFeed
  close: () => void
}

export function EditSmartFeedModal({ smartFeed, close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      title: smartFeed.title,
    },
    onSubmit: ({ value }) => {
      let title: string | null | undefined = value.title
      if (title === smartFeed.title) {
        title = undefined
      }

      if (title === undefined) {
        return close()
      }

      updateSmartFeed({
        id: smartFeed.id,
        body: {
          title,
        },
      })
    },
  })

  const { mutateAsync: updateSmartFeed, isPending } = useMutation(
    updateSmartFeedOptions(
      {
        onSuccess: async (data) => {
          form.reset()
          close()

          await context.queryClient.setQueryData(
            ['smartFeeds', smartFeed.id],
            data,
          )
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'smartFeeds'],
          })
        },
      },
      context.api,
    ),
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    form.reset()
  }, [form.reset, smartFeed.id])

  return (
    <Dialog.Content maxW="md" p={6}>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <Dialog.Title lineClamp={1}>Edit {smartFeed.title}</Dialog.Title>
        <Dialog.Description>Edit a smart feed's data.</Dialog.Description>
        <VStack alignItems="stretch" spaceY={4} mt={4}>
          <form.Field
            name="title"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z.string().min(1, "Title can't be empty"),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <Field.Root invalid={state.meta.errors.length > 0}>
                <Field.Label>Title</Field.Label>
                <Field.Input
                  value={state.value}
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <Field.HelperText>Smart feed title</Field.HelperText>
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
