import { useAPI } from '../../../lib/api-context'
import type { DetectedResponse } from '@colette/core'
import { detectFeedsOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import type { FC } from 'react'
import { z } from 'zod'
import { FormMessage } from '~/components/form'
import { Button } from '~/components/ui/button'
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const SearchStep: FC<{
  onNext: (res: DetectedResponse) => void
}> = (props) => {
  const api = useAPI()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => mutation.mutate(value),
  })

  const mutation = useMutation(
    detectFeedsOptions(api, {
      onSuccess: (res) => {
        form.reset()
        props.onNext(res)
      },
    }),
  )

  return (
    <>
      <DialogHeader>
        <DialogTitle>Search Feeds</DialogTitle>
        <DialogDescription>Find a feed by URL</DialogDescription>
      </DialogHeader>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field
          name="url"
          validators={{
            onSubmit: z.string().url('URL is not valid'),
          }}
        >
          {(field) => (
            <div className="space-y-1">
              <Label>URL</Label>
              <Input
                value={field.state.value}
                placeholder="https://example.com"
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <FormMessage>
                {field.state.meta.errors[0]?.toString()}
              </FormMessage>
            </div>
          )}
        </form.Field>
        <DialogFooter className="mt-6">
          <Button type="submit" disabled={mutation.isPending}>
            Search
          </Button>
        </DialogFooter>
      </form>
    </>
  )
}
