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
    <Dialog.Content>
      <Dialog.Title>Import Feeds</Dialog.Title>
      <Dialog.Description>
        Upload an OPML file to import feeds.
      </Dialog.Description>
      <form
        id="import-subscriptions"
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
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
          {(field) => {
            return (
              <Field.Root invalid={field.state.meta.errors.length !== 0}>
                <Field.Label>OPML file</Field.Label>
                <Field.Input
                  type="file"
                  value={field.state.value.name}
                  accept=".opml,text/xml,application/xml"
                  onChange={(e) => field.handleChange(e.target.files![0])}
                />
                <Field.ErrorText>{field.state.meta.errors[0]}</Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>
      </form>
      <Dialog.Footer>
        <Button
          form="import-subscriptions"
          disabled={importSubscriptions.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
