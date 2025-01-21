import type { Collection } from '@colette/core'
import { deleteCollectionOptions } from '@colette/query'
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

export const DeleteCollectionAlert: Component<{
  collection: Collection
  close: () => void
}> = (props) => {
  const api = useAPI()
  const navigate = useNavigate()
  const params = useParams<{ id?: string }>()
  const queryClient = useQueryClient()

  const mutation = createMutation(() =>
    deleteCollectionOptions(props.collection.id, api, queryClient, {
      onSuccess: () => {
        props.close()

        if (params.id === props.collection.id) {
          navigate('/collections')
        }
      },
    }),
  )

  return (
    <AlertDialogContent>
      <AlertDialogTitle>
        Delete <span class="text-orange-500">{props.collection.title}</span>
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
