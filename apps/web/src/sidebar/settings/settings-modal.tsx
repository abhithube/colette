import { useImportSubscriptionsMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import type { FC } from 'react'
import { FormMessage } from '~/components/form'
import { Button } from '~/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogTitle,
} from '~/components/ui/dialog'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const SettingsModal: FC<{ close: () => void }> = (props) => {
  const form = useForm({
    defaultValues: {
      file: undefined as unknown as File,
    },
    onSubmit: ({ value }) =>
      importSubscriptions.mutate(value.file, {
        onSuccess: () => {
          form.reset()
          props.close()
        },
      }),
  })

  const importSubscriptions = useImportSubscriptionsMutation()

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
            {(field) => (
              <div className="space-y-1">
                <Label>OPML file</Label>
                <Input
                  type="file"
                  value={field.state.value.name}
                  accept=".opml,text/xml,application/xml"
                  onChange={(e) => field.handleChange(e.target.files![0])}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
        </div>
        <div className="mt-4 flex justify-end">
          <Button disabled={importSubscriptions.isPending}>Submit</Button>
        </div>
      </form>
    </DialogContent>
  )
}
