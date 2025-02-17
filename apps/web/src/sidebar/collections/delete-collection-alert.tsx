import type { Collection } from '@colette/core'
import { useDeleteCollectionMutation } from '@colette/query'
import type { FC } from 'react'
import { useLocation, useParams } from 'wouter'
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
} from '~/components/ui/alert-dialog'

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
        Delete <span className="text-primary">{props.collection.title}</span>
      </AlertDialogTitle>
      <AlertDialogDescription>
        Are you sure you want to delete this collection and all of its
        bookmarks? This action cannot be undone.
      </AlertDialogDescription>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction
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
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  )
}
