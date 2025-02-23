import type { FeedDetected, FeedProcessed } from '@colette/core'
import { useDetectFeedsMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import type { FC } from 'react'
import { Favicon } from '~/components/favicon'
import { Button } from '~/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Label } from '~/components/ui/label'
import { RadioGroup, RadioGroupItem } from '~/components/ui/radio-group'

export const SelectStep: FC<{
  feeds: FeedDetected[]
  onNext: (feed: FeedProcessed) => void
  onBack: () => void
}> = (props) => {
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
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Select Feed</DialogTitle>
        <DialogDescription>Select a feed</DialogDescription>
      </DialogHeader>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field name="url">
          {(field) => (
            <RadioGroup
              value={field.state.value}
              onValueChange={field.handleChange}
            >
              {props.feeds.map((feed) => (
                <div key={feed.url} className="flex gap-4">
                  <RadioGroupItem
                    id={feed.url}
                    className="peer sr-only"
                    value={feed.url}
                  />
                  <Label
                    className="hover:bg-accent peer-data-[checked]:border-primary flex grow items-center gap-2 rounded-md border-2 p-4"
                    onClick={() => {
                      field.setValue(feed.url)
                    }}
                  >
                    <Favicon className="size-6" url={feed.url} />
                    <div className="flex flex-col gap-1">
                      <span className="font-semibold">{feed.title}</span>
                      <span className="text-muted-foreground text-sm">
                        {feed.url}
                      </span>
                    </div>
                  </Label>
                </div>
              ))}
            </RadioGroup>
          )}
        </form.Field>
        <DialogFooter className="mt-6">
          <Button variant="outline" onClick={props.onBack}>
            Back
          </Button>
          <Button type="submit" disabled={detectFeeds.isPending}>
            Select
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  )
}
