import { useAPI } from '../../lib/api-context'
import type { Collection } from '@colette/core'
import { deleteCollectionOptions } from '@colette/query'
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

export const DeleteCollectionAlert: FC<{
  collection: Collection
  close: () => void
}> = (props) => {
  const api = useAPI()
  const [, navigate] = useLocation()
  const params = useParams<{ id?: string }>()
  const queryClient = useQueryClient()

  const mutation = useMutation(
    deleteCollectionOptions(props.collection.id, api, {
      onSuccess: async () => {
        props.close()

        if (params.id === props.collection.id) {
          navigate('/collections')
        }

        await queryClient.invalidateQueries({
          queryKey: ['collections'],
        })
      },
    }),
  )

  return (
    <AlertDialogContent>
      <AlertDialogTitle>
        Delete <span className="text-orange-500">{props.collection.title}</span>
      </AlertDialogTitle>
      <AlertDialogDescription>
        Deleting a collection also deletes all bookmarks within the collection.
        Are you sure you want to delete this collection? This action cannot be
        undone.
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
