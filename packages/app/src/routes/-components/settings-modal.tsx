import { importOpmlOptions } from '@colette/query'
import { FormMessage } from '@colette/react-ui/components/form'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Input } from '@colette/react-ui/components/ui/input'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { Route } from '../_private'

type Props = {
  close: () => void
}

export function SettingsModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      file: undefined as unknown as File,
    },
    onSubmit: ({ value }) => importOpml(value.file),
  })

  const { mutateAsync: importOpml, isPending } = useMutation(
    importOpmlOptions(context.api, {
      onSuccess: async () => {
        form.reset()
        close()

        await context.queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })
      },
    }),
  )

  return (
    <DialogContent className="p-6">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <DialogTitle>Import Feeds</DialogTitle>
        <DialogDescription>
          Upload an OPML file to import feeds.
        </DialogDescription>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="file"
            validators={{
              onSubmit: ({ value }) => {
                if (!value) {
                  return 'Please select a valid OPML file'
                }
              },
            }}
          >
            {({ state, handleChange }) => (
              <div className="space-y-1">
                <Label>OPML file</Label>
                <Input
                  type="file"
                  accept=".opml,text/xml,application/xml"
                  onChange={(e) => handleChange(e.target.files![0])}
                />
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
            )}
          </form.Field>
        </div>
        <div className="mt-4 flex justify-end">
          <Button disabled={isPending}>Submit</Button>
        </div>
      </form>
    </DialogContent>
  )
}
