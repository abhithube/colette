import {
  Button,
  Dialog,
  Field,
  FileUpload,
  Flex,
  IconButton,
} from '@colette/components'
import { importOpmlOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { Trash2, X } from 'lucide-react'
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
    <Dialog.Content p={6}>
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
            <Field.Root mt={4} invalid={state.meta.errors.length > 0}>
              <FileUpload.Root
                accept={['application/xml', 'text/xml', '.opml']}
                onFileChange={(e) => handleChange(e.acceptedFiles[0])}
              >
                <FileUpload.Label>OPML file</FileUpload.Label>
                <FileUpload.Trigger asChild>
                  <Button variant="outline">Choose a file...</Button>
                </FileUpload.Trigger>
                <FileUpload.ItemGroup>
                  <FileUpload.Context>
                    {({ acceptedFiles }) =>
                      acceptedFiles.map((file) => (
                        <FileUpload.Item key={file.name} file={file}>
                          <FileUpload.ItemName />
                          <FileUpload.ItemSizeText />
                          <FileUpload.ItemDeleteTrigger asChild>
                            <IconButton
                              variant="ghost"
                              colorPalette="red"
                              size="sm"
                            >
                              <Trash2 />
                            </IconButton>
                          </FileUpload.ItemDeleteTrigger>
                        </FileUpload.Item>
                      ))
                    }
                  </FileUpload.Context>
                </FileUpload.ItemGroup>
                <FileUpload.HiddenInput />
              </FileUpload.Root>
              <Field.ErrorText>
                {state.meta.errors[0]?.toString()}
              </Field.ErrorText>
            </Field.Root>
          )}
        </form.Field>
        <Flex justify="end" mt={4}>
          <Button loading={isPending}>Submit</Button>
        </Flex>
      </form>
      <Dialog.CloseTrigger asChild position="absolute" top="2" right="2">
        <IconButton variant="ghost" size="sm">
          <X />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
