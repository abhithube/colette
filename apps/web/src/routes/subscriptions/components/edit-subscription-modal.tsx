import type { SubscriptionDetails } from '@colette/core'
import { useUpdateSubscriptionMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { type FC, useEffect } from 'react'
import { z } from 'zod'
import { FormDescription, FormMessage } from '~/components/form'
import { TagsInput } from '~/components/tags-input'
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

export const EditSubscriptionModal: FC<{
  details: SubscriptionDetails
  close: () => void
}> = (props) => {
  const form = useForm({
    defaultValues: {
      title: props.details.subscription.title,
      tags: props.details.tags?.map((tag) => tag.id) ?? [],
    },
    onSubmit: ({ value }) => {
      let tags: string[] | undefined = value.tags
      if (props.details.tags) {
        const current = props.details.tags
        if (
          tags?.length === current.length &&
          tags.every((id) => current.find((tag) => tag.id === id) !== undefined)
        ) {
          tags = undefined
        }
      } else if (tags.length === 0) {
        tags = undefined
      }

      if (
        value.title === props.details.subscription.title &&
        tags === undefined
      ) {
        return props.close()
      }

      updateSubscription.mutate(
        {
          title: value.title,
          tags,
        },
        {
          onSuccess: () => {
            form.reset()
            props.close()
          },
        },
      )
    },
  })

  const updateSubscription = useUpdateSubscriptionMutation(
    props.details.subscription.id,
  )

  useEffect(() => {
    form.reset()
  }, [form, props.details.subscription.id])

  return (
    <DialogContent className="max-w-md p-6">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <DialogHeader>
          <DialogTitle className="line-clamp-1">
            Edit{' '}
            <span className="text-primary">
              {props.details.subscription.title}
            </span>
          </DialogTitle>
          <DialogDescription>{"Edit a feed's data."}</DialogDescription>
        </DialogHeader>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="title"
            validators={{
              onBlur: z.string().min(1, "Title can't be empty"),
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Label>Title</Label>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormDescription>Title</FormDescription>
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
          <form.Field name="tags">
            {(field) => (
              <div className="space-y-1">
                <Label>Tags</Label>
                <TagsInput
                  state={field.state}
                  handleChange={field.handleChange}
                />
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button disabled={updateSubscription.isPending}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
