import { AlertDialog as BaseAlertDialog } from './ui/alert-dialog'
import { Dialog as BaseDialog } from './ui/dialog'
import { FC, useState } from 'react'

export const Dialog: FC<{
  children: (close: () => void) => React.ReactNode
}> = (props) => {
  const [isOpen, setOpen] = useState(false)

  return (
    <BaseDialog open={isOpen} onOpenChange={setOpen}>
      {props.children(() => setOpen(false))}
    </BaseDialog>
  )
}
export const AlertDialog: FC<{
  children: (close: () => void) => React.ReactNode
}> = (props) => {
  const [isOpen, setOpen] = useState(false)

  return (
    <BaseAlertDialog open={isOpen} onOpenChange={setOpen}>
      {props.children(() => setOpen(false))}
    </BaseAlertDialog>
  )
}
