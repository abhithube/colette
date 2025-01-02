import type { FeedDetected, FeedProcessed } from '@colette/core'
import { detectFeedsOptions } from '@colette/solid-query'
import { createForm } from '@tanstack/solid-form'
import { createMutation } from '@tanstack/solid-query'
import { type Component, For } from 'solid-js'
import { Favicon } from '~/components/favicon'
import { Button } from '~/components/ui/button'
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Label } from '~/components/ui/label'
import { RadioGroup, RadioGroupItem } from '~/components/ui/radio-group'
import { useAPI } from '~/lib/api-context'

export const SelectStep: Component<{
  feeds: FeedDetected[]
  onNext: (feed: FeedProcessed) => void
  onBack: () => void
}> = (props) => {
  const form = createForm(() => ({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => detectFeeds(value),
  }))

  const { mutateAsync: detectFeeds, isPending } = createMutation(() =>
    detectFeedsOptions(
      {
        onSuccess: (res) => {
          form.reset()

          if ('link' in res) {
            props.onNext(res)
          }
        },
      },
      useAPI(),
    ),
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
              value={field().state.value}
              onChange={field().handleChange}
            >
              <For each={props.feeds}>
                {(item) => (
                  <div class="flex gap-4">
                    <RadioGroupItem
                      id={item.url}
                      class="peer sr-only"
                      value={item.url}
                    />
                    <Label
                      class="flex grow items-center gap-2 rounded-md border-2 p-4 hover:bg-accent peer-data-[checked]:border-primary"
                      for={item.url}
                      onClick={() => {
                        field().setValue(item.url)
                      }}
                    >
                      <Favicon class="size-6" url={item.url} />
                      <div class="flex flex-col gap-1">
                        <span class="font-semibold">{item.title}</span>
                        <span class="text-muted-foreground text-sm">
                          {item.url}
                        </span>
                      </div>
                    </Label>
                  </div>
                )}
              </For>
            </RadioGroup>
          )}
        </form.Field>
        <DialogFooter class="mt-6">
          <Button variant="outline" onClick={props.onBack}>
            Back
          </Button>
          <Button type="submit" disabled={isPending}>
            Select
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
