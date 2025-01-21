import { useAPI } from '../../../lib/api-context'
import type { FeedProcessed } from '@colette/core'
import { createFeedOptions } from '@colette/query'
import { FormMessage } from '@colette/react-ui/components/form'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Input } from '@colette/react-ui/components/ui/input'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { RotateCcw } from 'lucide-react'
import type { FC } from 'react'
import { useLocation } from 'wouter'
import { z } from 'zod'

export const EditStep: FC<{
  feed: FeedProcessed
  onClose: () => void
  onBack: () => void
}> = (props) => {
  const api = useAPI()
  const [, navigate] = useLocation()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      title: props.feed.title,
    },
    onSubmit: ({ value }) => {
      let title: string | undefined = value.title
      if (title === props.feed.title) {
        title = undefined
      }

      mutation.mutate({
        url: props.feed.link,
        title,
      })
    },
  })

  const mutation = useMutation(
    createFeedOptions(api, {
      onSuccess: async (feed) => {
        form.reset()

        await queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })

        navigate(`/feeds/${feed.id}`)

        props.onClose()
      },
    }),
  )

  return (
    <>
      <DialogHeader>
        <DialogTitle>Edit Feed</DialogTitle>
        <DialogDescription>
          Modify a feed&apos;s metadata before subscribing to it
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
                <Button
                  variant="outline"
                  onClick={() => field.setValue(props.feed.title)}
                >
                  <RotateCcw />
                </Button>
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
          <Button type="submit" disabled={mutation.isPending}>
            Submit
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
