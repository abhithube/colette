import type { DetectedResponse } from '@colette/core'
import { useDetectFeedsMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { z } from 'zod'

export const SearchStep = (props: {
  onNext: (res: DetectedResponse) => void
}) => {
  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) =>
      detectFeeds.mutate(value, {
        onSuccess: (res) => {
          form.reset()
          props.onNext(res)
        },
      }),
  })

  const detectFeeds = useDetectFeedsMutation()

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Search Feeds</Dialog.Title>
        <Dialog.Description>Find a feed by URL</Dialog.Description>
      </Dialog.Header>
      <form
        id="search-step"
        className="space-y-4"
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
          {(field) => {
            return (
              <Field.Root invalid={field.state.meta.errors.length !== 0}>
                <Field.Label>URL</Field.Label>
                <Field.Input
                  value={field.state.value}
                  placeholder="https://example.com"
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.message}
                </Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>
      </form>
      <Dialog.Footer>
        <Button
          form="search-step"
          type="submit"
          disabled={detectFeeds.isPending}
        >
          Search
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
