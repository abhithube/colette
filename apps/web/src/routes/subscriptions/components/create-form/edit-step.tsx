import { Feed } from '@colette/core'
import { useCreateSubscriptionMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'

export const EditStep = (props: {
  feed: Feed
  onClose: () => void
  onBack: () => void
}) => {
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
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Edit Feed</Dialog.Title>
        <Dialog.Description>
          {"Modify a feed's metadata before subscribing to it"}
        </Dialog.Description>
      </Dialog.Header>
      <form
        id="edit-step"
        className="space-y-4"
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
          {(field) => {
            return (
              <Field.Root invalid={field.state.meta.errors.length !== 0}>
                <Field.Label>Title</Field.Label>
                <Field.Input
                  value={field.state.value}
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
      <Dialog.Footer>
        <Button variant="outline" onClick={props.onBack}>
          Back
        </Button>
        <Button form="edit-step" type="submit" disabled={createdFeed.isPending}>
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
