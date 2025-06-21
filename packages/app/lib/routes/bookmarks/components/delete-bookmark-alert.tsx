import { Bookmark } from '@colette/core/types'
import { useDeleteBookmarkMutation } from '@colette/query'
import { Button, Dialog } from '@colette/ui'

export const DeleteBookmarkAlert = (props: {
  bookmark: Bookmark
  close: () => void
}) => {
  const deleteBookmark = useDeleteBookmarkMutation(props.bookmark.id)

  function onDelete() {
    deleteBookmark.mutate(undefined, {
      onSuccess: () => {
        props.close()
      },
    })
  }

  return (
    <Dialog.Content>
      <Dialog.Title className="line-clamp-1">
        Delete <span className="text-primary">{props.bookmark.title}</span>
      </Dialog.Title>
      <Dialog.Description>
        Are you sure you want to delete this bookmark? This action cannot be
        undone.
      </Dialog.Description>
      <Dialog.Footer>
        <Button variant="secondary" onClick={props.close}>
          Cancel
        </Button>
        <Button disabled={deleteBookmark.isPending} onClick={onDelete}>
          Confirm
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
