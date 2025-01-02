import type { DetectedResponse } from '@colette/core'
import { detectFeedsOptions } from '@colette/solid-query'
import { createForm } from '@tanstack/solid-form'
import { createMutation } from '@tanstack/solid-query'
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

export const SearchStep: Component<{
  onNext: (res: DetectedResponse) => void
}> = (props) => {
  const form = createForm(() => ({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => detectFeeds(value),
  }))

  const { mutateAsync: detectFeeds, isPending } = createMutation(() =>
    detectFeedsOptions(
      {
        onSuccess: (res) => {
          form.reset()
          props.onNext(res)
        },
      },
      useAPI(),
    ),
  )

  return (
    <>
      <DialogHeader>
        <DialogTitle>Search Feeds</DialogTitle>
        <DialogDescription>Find a feed by URL</DialogDescription>
      </DialogHeader>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field
          name="url"
          validators={{
            onSubmit: z.string().url('URL is not valid'),
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
              <TextFieldLabel>URL</TextFieldLabel>
              <TextFieldInput placeholder="https://example.com" />
              <TextFieldErrorMessage>
                {field().state.meta.errors[0]?.toString()}
              </TextFieldErrorMessage>
            </TextField>
          )}
        </form.Field>
        <DialogFooter class="mt-6">
          <Button type="submit" disabled={isPending}>
            Search
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
