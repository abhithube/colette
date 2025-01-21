import { useAPI } from '../../../lib/api-context'
import type { FeedDetected, FeedProcessed } from '@colette/core'
import { detectFeedsOptions } from '@colette/query'
import { Favicon } from '@colette/react-ui/components/favicon'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Label } from '@colette/react-ui/components/ui/label'
import {
  RadioGroup,
  RadioGroupItem,
} from '@colette/react-ui/components/ui/radio-group'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import type { FC } from 'react'

export const SelectStep: FC<{
  feeds: FeedDetected[]
  onNext: (feed: FeedProcessed) => void
  onBack: () => void
}> = (props) => {
  const api = useAPI()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => mutation.mutate(value),
  })

  const mutation = useMutation(
    detectFeedsOptions(api, {
      onSuccess: (res) => {
        form.reset()

        if ('link' in res) {
          props.onNext(res)
        }
      },
    }),
  )

  return (
    <>
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
          <Button type="submit" disabled={mutation.isPending}>
            Select
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
