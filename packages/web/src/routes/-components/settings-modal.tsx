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
import { importOpmlOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { zfd } from 'zod-form-data'
import { Route } from '../_private'

const formSchema = z.object({
  file: zfd.file(),
})

type Props = {
  close: () => void
}

export function SettingsModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const { mutateAsync: importFeeds, isPending } = useMutation(
    importOpmlOptions(
      {
        onSuccess: async () => {
          form.reset()
          close()

          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
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
          onSubmit={form.handleSubmit(async (data) => importFeeds(data.file))}
        >
          <DialogHeader>
            <DialogTitle>Import Feeds</DialogTitle>
            <DialogDescription>
              Upload an OPML file to import feeds.
            </DialogDescription>
          </DialogHeader>
          <FormField
            control={form.control}
            name="file"
            render={({ field }) => (
              <FormItem>
                <FormLabel>File</FormLabel>
                <Input
                  type="file"
                  onChange={(ev) =>
                    field.onChange(ev.target.files ? ev.target.files[0] : null)
                  }
                />
                <FormDescription>OPML file to upload</FormDescription>
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
