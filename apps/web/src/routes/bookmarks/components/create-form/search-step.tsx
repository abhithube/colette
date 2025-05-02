import type { BookmarkScraped } from '@colette/core'
import { scrapeBookmarkFormOptions } from '@colette/form'
import { useScrapeBookmarkMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi } from '@tanstack/react-router'

const routeApi = getRouteApi('/layout/stash')

export const SearchStep = (props: {
  formId: string
  onNext: (scraped: BookmarkScraped) => void
}) => {
  const context = routeApi.useRouteContext()

  const form = useForm({
    ...scrapeBookmarkFormOptions(),
    onSubmit: ({ value, formApi }) =>
      scrapeBookmark.mutate(value, {
        onSuccess: (scraped) => {
          formApi.reset()

          props.onNext(scraped)
        },
      }),
  })

  const scrapeBookmark = useScrapeBookmarkMutation(context.api)

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
