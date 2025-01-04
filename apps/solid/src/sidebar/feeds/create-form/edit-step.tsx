import type { FeedProcessed } from '@colette/core'
import { createFeedOptions } from '@colette/solid-query'
import { useNavigate } from '@solidjs/router'
import { createForm } from '@tanstack/solid-form'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import RotateCcw from 'lucide-solid/icons/rotate-ccw'
import type { Component } from 'solid-js'
import { z } from 'zod'
import { Button } from '~/components/ui/button'
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '~/components/ui/text-field'
import { useAPI } from '~/lib/api-context'

export const EditStep: Component<{
  feed: FeedProcessed
  onClose: () => void
  onBack: () => void
}> = (props) => {
  const navigate = useNavigate()

  const form = createForm(() => ({
    defaultValues: {
      title: props.feed.title,
    },
    onSubmit: ({ value }) => {
      let title: string | undefined = value.title
      if (title === props.feed.title) {
        title = undefined
      }

      mutation.mutate({
        url: props.feed.link,
        title,
      })
    },
  }))

  const queryClient = useQueryClient()

  const mutation = createMutation(() =>
    createFeedOptions(
      {
        onSuccess: async (feed) => {
          form.reset()

          await queryClient.invalidateQueries({
            queryKey: ['feeds'],
          })

          navigate(`/feeds/${feed.id}`)

          props.onClose()
        },
      },
      useAPI(),
    ),
  )

  return (
    <>
      <DialogHeader>
        <DialogTitle>Edit Feed</DialogTitle>
        <DialogDescription>
          Modify a feed's metadata before subscribing to it
        </DialogDescription>
      </DialogHeader>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field
          name="title"
          validators={{
            onSubmit: z.string().min(1, 'Title cannot be empty'),
          }}
        >
          {(field) => (
            <TextField
              class="grow space-y-1"
              value={field().state.value}
              onChange={field().handleChange}
              validationState={
                field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
              }
            >
              <TextFieldLabel>Title</TextFieldLabel>
              <div class="flex gap-2">
                <TextFieldInput />
                <Button
                  variant="outline"
                  onClick={() => field().setValue(props.feed.title)}
                >
                  <RotateCcw />
                </Button>
              </div>
              <TextFieldErrorMessage>
                {field().state.meta.errors[0]?.toString()}
              </TextFieldErrorMessage>
            </TextField>
          )}
        </form.Field>
        <DialogFooter class="mt-6">
          <Button variant="outline" onClick={props.onBack}>
            Back
          </Button>
          <Button type="submit" disabled={mutation.isPending}>
            Submit
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
