import { useAPI } from '../../lib/api-context'
import { importOpmlOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
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
  const api = useAPI()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      file: undefined as unknown as File,
    },
    onSubmit: ({ value }) => importOpml(value.file),
  })

  const { mutateAsync: importOpml, isPending } = useMutation(
    importOpmlOptions(api, queryClient, {
      onSuccess: () => {
        form.reset()
        props.close()
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
