import { FeedDetected } from '@colette/core/types'
import { detectFeedsFormOptions } from '@colette/form'
import { useDetectFeedsMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const SearchStep = (props: {
  formId: string
  onNext: (detected: FeedDetected[]) => void
}) => {
  const form = useForm({
    ...detectFeedsFormOptions(),
    onSubmit: ({ value, formApi }) =>
      detectFeeds.mutate(value, {
        onSuccess: (detected) => {
          formApi.reset()

          props.onNext(detected)
        },
      }),
  })

  const detectFeeds = useDetectFeedsMutation()

  return (
    <form
      id={props.formId}
      className="space-y-4"
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <form.Field name="url">
        {(field) => {
          const errors = field.state.meta.errors

          return (
            <Field.Root invalid={errors.length !== 0}>
              <Field.Label>URL</Field.Label>
              <Field.Input
                value={field.state.value}
                placeholder="https://example.com"
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
            </Field.Root>
          )
        }}
      </form.Field>
    </form>
  )
}
