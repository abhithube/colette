import { useCreateCollectionMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { Plus } from 'lucide-react'
import { type FC, useState } from 'react'
import { useLocation } from 'wouter'
import { z } from 'zod'
import { FormMessage } from '~/components/form'
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
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'
import { SidebarGroupAction } from '~/components/ui/sidebar'

export const CreateCollectionModal: FC = () => {
  const [, navigate] = useLocation()

  const [isOpen, setOpen] = useState(false)

  const form = useForm({
    defaultValues: {
      title: '',
    },
    onSubmit: ({ value }) =>
      createCollection.mutate(value, {
        onSuccess: (collection) => {
          form.reset()
          setOpen(false)

          navigate(`/collections/${collection.id}`)
        },
      }),
  })

  const createCollection = useCreateCollectionMutation()

  return (
    <Dialog open={isOpen} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <SidebarGroupAction>
          <Plus />
        </SidebarGroupAction>
      </DialogTrigger>
      <DialogContent className="gap-6">
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
              <div className="space-y-1">
                <Label>Title</Label>
                <Input
                  value={field.state.value}
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
          <DialogFooter className="mt-6">
            <Button type="submit" disabled={createCollection.isPending}>
              Submit
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
