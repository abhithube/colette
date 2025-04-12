import { useImportSubscriptionsMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const SettingsModal = (props: { close: () => void }) => {
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
    <Dialog.Content className="p-6">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <Dialog.Title>Import Feeds</Dialog.Title>
        <Dialog.Description>
          Upload an OPML file to import feeds.
        </Dialog.Description>
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
              <Field.Root className="space-y-1">
                <Field.Label>OPML file</Field.Label>
                <Field.Input
                  type="file"
                  value={field.state.value.name}
                  accept=".opml,text/xml,application/xml"
                  onChange={(e) => field.handleChange(e.target.files![0])}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </div>
        <div className="mt-4 flex justify-end">
          <Button disabled={importSubscriptions.isPending}>Submit</Button>
        </div>
      </form>
    </Dialog.Content>
  )
}
