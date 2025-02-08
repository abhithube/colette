import { useAPI } from '../../lib/api-context'
import type { Feed } from '@colette/core'
import { deleteFeedOptions } from '@colette/query'
import {
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from '@colette/react-ui/components/ui/alert-dialog'
import { Button } from '@colette/react-ui/components/ui/button'
import { DialogFooter } from '@colette/react-ui/components/ui/dialog'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { FC } from 'react'
import { useLocation, useParams } from 'wouter'

export const UnsubscribeAlert: FC<{
  feed: Feed
  close: () => void
}> = (props) => {
  const api = useAPI()
  const [, navigate] = useLocation()
  const queryClient = useQueryClient()

  const params = useParams<{ id?: string }>()

  const mutation = useMutation(
    deleteFeedOptions(props.feed.id, api, queryClient, {
      onSuccess: () => {
        props.close()

        if (params.id === props.feed.id) {
          navigate('/feeds')
        }
      },
    }),
  )

  return (
    <AlertDialogContent>
      <AlertDialogTitle>
        Unsubscribe from{' '}
        <span className="text-orange-500">{props.feed.title}</span>
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
