import type { FeedDetected } from '@colette/core'
import { useDetectFeedsMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi } from '@tanstack/react-router'
import { z } from 'zod'

const routeApi = getRouteApi('/layout/subscriptions/')

export const SearchStep = (props: {
  formId: string
  onNext: (detected: FeedDetected[]) => void
}) => {
  const context = routeApi.useRouteContext()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) =>
      detectFeeds.mutate(value, {
        onSuccess: (detected) => {
          form.reset()
          props.onNext(detected)
        },
      }),
  })

  const detectFeeds = useDetectFeedsMutation(context.api)

  return (
    <form
      id={props.formId}
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
  )
}
