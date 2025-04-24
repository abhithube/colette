import type { BookmarkScraped } from '@colette/core'
import { useScrapeBookmarkMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { z } from 'zod'

export const SearchStep = (props: {
  formId: string
  onNext: (scraped: BookmarkScraped) => void
}) => {
  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) =>
      scrapeBookmark.mutate(value, {
        onSuccess: (scraped) => {
          form.reset()
          props.onNext(scraped)
        },
      }),
  })

  const scrapeBookmark = useScrapeBookmarkMutation()

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
