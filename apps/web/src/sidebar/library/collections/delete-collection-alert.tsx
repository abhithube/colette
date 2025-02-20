import type { Collection } from '@colette/core'
import { useDeleteCollectionMutation } from '@colette/query'
import type { FC } from 'react'
import { useParams } from 'wouter'
import { navigate } from 'wouter/use-browser-location'
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
  const params = useParams<{ id?: string }>()

  const deleteCollection = useDeleteCollectionMutation(props.collection.id)

  function onDelete() {
    deleteCollection.mutate(undefined, {
      onSuccess: () => {
        props.close()

        if (params.id === props.collection.id) {
          navigate('/collections')
        }
      },
    })
  }

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
          onClick={onDelete}
        >
          Confirm
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  )
}
