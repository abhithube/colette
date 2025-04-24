import type { Feed, FeedDetected } from '@colette/core'
import { useScrapeFeedMutation } from '@colette/query'
import { Field, RadioGroup, Favicon } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const SelectStep = (props: {
  formId: string
  feeds: FeedDetected[]
  onNext: (feed: Feed) => void
}) => {
  const form = useForm({
    defaultValues: {
      url: props.feeds[0].url,
    },
    onSubmit: ({ value }) =>
      scrape.mutate(value, {
        onSuccess: (feed) => {
          form.reset()
          props.onNext(feed)
        },
      }),
  })

  const scrape = useScrapeFeedMutation()

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
          return (
            <RadioGroup.Root
              value={field.state.value}
              onValueChange={(details) => {
                if (details.value) {
                  field.handleChange(details.value)
                }
              }}
            >
              {props.feeds.map((feed) => {
                return (
                  <Field.Root key={feed.url} className="gap-4">
                    <RadioGroup.Item
                      id={feed.url}
                      className="peer sr-only"
                      value={feed.url}
                    />
                    <Field.Label
                      className="hover:bg-accent peer-data-[checked]:border-primary flex-1 rounded-md border-2 p-4"
                      onClick={() => {
                        field.setValue(feed.url)
                      }}
                    >
                      <Favicon className="size-6" src={feed.url} />
                      <div className="flex flex-col gap-1">
                        <span className="font-semibold">{feed.title}</span>
                        <span className="text-muted-foreground text-sm">
                          {feed.url}
                        </span>
                      </div>
                    </Field.Label>
                  </Field.Root>
                )
              })}
            </RadioGroup.Root>
          )
        }}
      </form.Field>
    </form>
  )
}
