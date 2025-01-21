import { useAPI } from '../../../lib/api-context'
import { createFeedOptions } from '@colette/query'
import { FormDescription, FormMessage } from '@colette/react-ui/components/form'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Input } from '@colette/react-ui/components/ui/input'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { FC } from 'react'
import { useLocation } from 'wouter'
import { z } from 'zod'

export const SubscribeModal: FC<{ close: () => void }> = (props) => {
  const api = useAPI()
  const [, navigate] = useLocation()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: ({ value }) => createFeed(value),
  })

  const { mutateAsync: createFeed, isPending } = useMutation(
    createFeedOptions(api, {
      onSuccess: async (data) => {
        form.reset()
        props.close()

        await queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })

        navigate(`/feeds/${data.id}`)
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
        <DialogTitle>Add Feed</DialogTitle>
        <DialogDescription>
          Subscribe to a RSS or Atom feed and receive the latest updates.
        </DialogDescription>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="url"
            validators={{
              onBlur: z.string().url('Please enter a valid URL'),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <div className="space-y-1">
                <Label>URL</Label>
                <Input
                  placeholder="https://example.com"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <FormDescription>URL of the RSS or Atom Feed</FormDescription>
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button disabled={isPending}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
