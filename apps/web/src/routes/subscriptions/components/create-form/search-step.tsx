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
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <div className="flex flex-col items-stretch gap-4">
          <form.Field
            name="url"
            validators={{
              onSubmit: z.string().url('URL is not valid'),
            }}
          >
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>URL</Field.Label>
                <Field.Input
                  value={field.state.value}
                  placeholder="https://example.com"
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </div>
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
