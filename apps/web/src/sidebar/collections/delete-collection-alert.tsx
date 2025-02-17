import type { Collection } from '@colette/core'
import { useDeleteCollectionMutation } from '@colette/query'
import type { FC } from 'react'
import { useLocation, useParams } from 'wouter'
import {
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from '~/components/ui/alert-dialog'
import { Button } from '~/components/ui/button'
import { DialogFooter } from '~/components/ui/dialog'

export const DeleteCollectionAlert: FC<{
  collection: Collection
  close: () => void
}> = (props) => {
  const [, navigate] = useLocation()
  const params = useParams<{ id?: string }>()

  const deleteCollection = useDeleteCollectionMutation(props.collection.id)

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
        <Button
          disabled={deleteCollection.isPending}
          onClick={() =>
            deleteCollection.mutate(undefined, {
              onSuccess: () => {
                props.close()

                if (params.id === props.collection.id) {
                  navigate('/collections')
                }
              },
            })
          }
        >
          Confirm
        </Button>
      </DialogFooter>
    </AlertDialogContent>
  )
}
