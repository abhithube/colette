import type { Feed } from '@colette/core'
import { useCreateSubscriptionMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import type { FC } from 'react'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'
import { FormMessage } from '~/components/form'
import { Button } from '~/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const EditStep: FC<{
  feed: Feed
  onClose: () => void
  onBack: () => void
}> = (props) => {
  const form = useForm({
    defaultValues: {
      title: props.feed.title,
    },
    onSubmit: ({ value }) => {
      createdFeed.mutate(
        {
          title: value.title,
          feedId: props.feed.id,
        },
        {
          onSuccess: (feed) => {
            form.reset()
            props.onClose()

            navigate(`/feeds/${feed.id}`)
          },
        },
      )
    },
  })

  const createdFeed = useCreateSubscriptionMutation()

  return (
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Edit Feed</DialogTitle>
        <DialogDescription>
          {"Modify a feed's metadata before subscribing to it"}
        </DialogDescription>
      </DialogHeader>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field
          name="title"
          validators={{
            onSubmit: z.string().min(1, 'Title cannot be empty'),
          }}
        >
          {(field) => (
            <div className="space-y-1">
              <Label>Title</Label>
              <div className="flex gap-2">
                <Input
                  value={field.state.value}
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
              </div>
              <FormMessage>
                {field.state.meta.errors[0]?.toString()}
              </FormMessage>
            </div>
          )}
        </form.Field>
        <DialogFooter className="mt-6">
          <Button variant="outline" onClick={props.onBack}>
            Back
          </Button>
          <Button type="submit" disabled={createdFeed.isPending}>
            Submit
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  )
}
