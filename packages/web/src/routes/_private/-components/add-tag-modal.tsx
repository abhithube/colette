import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Form,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { createTagOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../_private'

const formSchema = z.object({
  title: z.string().min(1),
})

type Props = {
  close: () => void
}

export function AddTagModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: '',
    },
  })

  const navigate = useNavigate()

  const { mutateAsync: createTag, isPending } = useMutation(
    createTagOptions(
      {
        onSuccess: () => {
          form.reset()
          close()

          navigate({
            to: '/bookmarks/stash',
          })
        },
      },
      context.api,
    ),
  )

  return (
    <DialogContent>
      <Form {...form}>
        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((data) => createTag(data))}
        >
          <DialogHeader>
            <DialogTitle>Add Tag</DialogTitle>
            <DialogDescription>Add a tag.</DialogDescription>
          </DialogHeader>
          <FormField
            control={form.control}
            name="title"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Title</FormLabel>
                <Input {...field} />
                <FormDescription>Title of the tag</FormDescription>
              </FormItem>
            )}
          />
          <DialogFooter>
            <Button disabled={isPending}>
              {isPending && (
                <Icon className="mr-2 animate-spin" value={Loader2} />
              )}
              Submit
            </Button>
          </DialogFooter>
        </form>
      </Form>
    </DialogContent>
  )
}
