import type { Feed } from '@colette/core'
import { deleteFeedOptions } from '@colette/solid-query'
import { useNavigate, useParams } from '@solidjs/router'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import type { Component } from 'solid-js'
import {
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from '~/components/ui/alert-dialog'
import { Button } from '~/components/ui/button'
import { DialogFooter } from '~/components/ui/dialog'
import { useAPI } from '~/lib/api-context'

export const UnsubscribeAlert: Component<{
  feed: Feed
  close: () => void
}> = (props) => {
  const queryClient = useQueryClient()

  const navigate = useNavigate()

  const params = useParams<{ id?: string }>()

  const mutation = createMutation(() =>
    deleteFeedOptions(
      props.feed.id,
      {
        onSuccess: async () => {
          props.close()

          if (params.id === props.feed.id) {
            navigate('/feeds')
          }

          await queryClient.invalidateQueries({
            queryKey: ['feeds'],
          })
        },
      },
      useAPI(),
    ),
  )

  return (
    <AlertDialogContent>
      <AlertDialogTitle>
        Unsubscribe from{' '}
        <span class="text-orange-500">
          {props.feed.title ?? props.feed.originalTitle}
        </span>
      </AlertDialogTitle>
      <AlertDialogDescription>
        Are you sure you want to unsubscribe? This action cannot be undone.
      </AlertDialogDescription>
      <DialogFooter>
        <Button variant="outline" onClick={() => props.close()}>
          Close
        </Button>
        <Button disabled={mutation.isPending} onClick={() => mutation.mutate()}>
          Confirm
        </Button>
      </DialogFooter>
    </AlertDialogContent>
  )
}
