import { createProfileOptions } from '@colette/query'
import { Button, Dialog, Field, Flex, VStack } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { z } from 'zod'
import { Route } from '../_private'

type Props = {
  back: () => void
}

export function AddProfileForm({ back }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      title: '',
      imageUrl: undefined as string | undefined,
    },
    onSubmit: ({ value }) => createProfile(value),
  })

  const { mutateAsync: createProfile, isPending } = useMutation(
    createProfileOptions(
      {
        onSuccess: async () => {
          form.reset()

          await context.queryClient.invalidateQueries({
            queryKey: ['profiles'],
          })

          back()
        },
      },
      context.api,
    ),
  )

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Dialog.Title>Add Profile</Dialog.Title>
      <Dialog.Description>Add a user profile.</Dialog.Description>
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
              <Field.HelperText>Profile title</Field.HelperText>
              <Field.ErrorText>
                {state.meta.errors[0]?.toString()}
              </Field.ErrorText>
            </Field.Root>
          )}
        </form.Field>
        <form.Field
          name="imageUrl"
          validatorAdapter={zodValidator()}
          validators={{
            onBlur: z.string().url('Please enter a valid URL').optional(),
          }}
        >
          {({ state, handleChange, handleBlur }) => (
            <Field.Root
              defaultValue={state.value}
              invalid={state.meta.errors.length > 0}
            >
              <Field.Label>Image URL</Field.Label>
              <Field.Input
                placeholder="https://www.website.com"
                onChange={(e) => handleChange(e.target.value)}
                onBlur={handleBlur}
              />
              <Field.HelperText>URL of the profile image</Field.HelperText>
              <Field.ErrorText>
                {state.meta.errors[0]?.toString()}
              </Field.ErrorText>
            </Field.Root>
          )}
        </form.Field>
        <Flex justify="end" gap={2}>
          <Button variant="ghost" onClick={back}>
            Back
          </Button>
          <Button loading={isPending} type="submit">
            Submit
          </Button>
        </Flex>
      </VStack>
    </form>
  )
}
