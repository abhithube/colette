import type { Feed, FeedDetected } from '@colette/core'
import { useDetectFeedsMutation } from '@colette/query'
import { Button, Dialog, Field, RadioGroup, Favicon } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const SelectStep = (props: {
  feeds: FeedDetected[]
  onNext: (feed: Feed) => void
  onBack: () => void
}) => {
  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) =>
      detectFeeds.mutate(value, {
        onSuccess: (res) => {
          form.reset()

          if ('link' in res) {
            props.onNext(res)
          }
        },
      }),
  })

  const detectFeeds = useDetectFeedsMutation()

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Select Feed</Dialog.Title>
        <Dialog.Description>Select a feed</Dialog.Description>
      </Dialog.Header>
      <form
        id="select-step"
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
                    <Field.Root key={feed.url} className="flex gap-4">
                      <RadioGroup.Item
                        id={feed.url}
                        className="peer sr-only"
                        value={feed.url}
                      />
                      <Field.Label
                        className="hover:bg-accent peer-data-[checked]:border-primary flex grow items-center gap-2 rounded-md border-2 p-4"
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
      <Dialog.Footer>
        <Button variant="outline" onClick={props.onBack}>
          Back
        </Button>
        <Button
          form="select-step"
          type="submit"
          disabled={detectFeeds.isPending}
        >
          Select
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
