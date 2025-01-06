import { createCollectionOptions } from '@colette/solid-query'
import { useNavigate } from '@solidjs/router'
import { createForm } from '@tanstack/solid-form'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import Plus from 'lucide-solid/icons/plus'
import { type Component, createSignal } from 'solid-js'
import { z } from 'zod'
import { Button } from '~/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '~/components/ui/dialog'
import { SidebarGroupAction } from '~/components/ui/sidebar'
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '~/components/ui/text-field'
import { useAPI } from '~/lib/api-context'

export const CreateCollectionModal: Component = () => {
  const api = useAPI()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [open, setOpen] = createSignal(false)

  const form = createForm(() => ({
    defaultValues: {
      title: '',
    },
    onSubmit: ({ value }) => mutation.mutate(value),
  }))

  const mutation = createMutation(() =>
    createCollectionOptions(api, {
      onSuccess: async (collection) => {
        form.reset()

        await queryClient.invalidateQueries({
          queryKey: ['collections'],
        })

        navigate(`/collections/${collection.id}`)

        setOpen(false)
      },
    }),
  )

  return (
    <Dialog open={open()} onOpenChange={setOpen}>
      <DialogTrigger as={SidebarGroupAction}>
        <Plus />
      </DialogTrigger>
      <DialogContent class="gap-6">
        <DialogHeader>
          <DialogTitle>Add Collection</DialogTitle>
          <DialogDescription>
            Create a new collection of bookmarks.
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
              <TextField
                class="grow space-y-1"
                value={field().state.value}
                onChange={field().handleChange}
                validationState={
                  field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
                }
              >
                <TextFieldLabel>Title</TextFieldLabel>
                <TextFieldInput />
                <TextFieldErrorMessage>
                  {field().state.meta.errors[0]?.toString()}
                </TextFieldErrorMessage>
              </TextField>
            )}
          </form.Field>
          <DialogFooter class="mt-6">
            <Button type="submit" disabled={mutation.isPending}>
              Submit
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
