import { cn } from './utils'
import { Dialog, type HTMLArkProps, Portal, ark } from '@ark-ui/react'
import { X } from 'lucide-react'

export type RootProps = Dialog.RootProps

export const Root = Dialog.Root

export type ContextProps = Dialog.ContextProps

export const Context = Dialog.Context

export type TriggerProps = Dialog.TriggerProps

export const Trigger = Dialog.Trigger

export type BackdropProps = Dialog.BackdropProps

export const Backdrop = ({ className, ...props }: BackdropProps) => {
  return (
    <Dialog.Backdrop
      className={cn(
        'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50',
        className,
      )}
      {...props}
    />
  )
}

export type ContentProps = Dialog.ContentProps

export const Content = ({ className, children, ...props }: ContentProps) => {
  return (
    <Portal>
      <Backdrop />
      <Dialog.Positioner>
        <Dialog.Content
          className={cn(
            'bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg',
            className,
          )}
          {...props}
        >
          {children}
          <Dialog.CloseTrigger className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4">
            <X className="size-4" />
            <span className="sr-only">Close</span>
          </Dialog.CloseTrigger>
        </Dialog.Content>
      </Dialog.Positioner>
    </Portal>
  )
}

export type HeaderProps = HTMLArkProps<'div'>

export const Header = ({ className, ...props }: HeaderProps) => {
  return (
    <ark.div
      data-scope="dialog"
      data-part="header"
      className={cn('flex flex-col gap-2 text-center sm:text-left', className)}
      {...props}
    />
  )
}

export type FooterProps = HTMLArkProps<'div'>

export const Footer = ({ className, ...props }: FooterProps) => {
  return (
    <ark.div
      data-scope="dialog"
      data-part="footer"
      className={cn(
        'flex flex-col-reverse gap-2 sm:flex-row sm:justify-end',
        className,
      )}
      {...props}
    />
  )
}

export type TitleProps = Dialog.TitleProps

export const Title = ({ className, ...props }: TitleProps) => {
  return (
    <Dialog.Title
      className={cn('text-lg leading-none font-semibold', className)}
      {...props}
    />
  )
}

export type DescriptionProps = Dialog.DescriptionProps

export const Description = ({ className, ...props }: DescriptionProps) => {
  return (
    <Dialog.Description
      className={cn('text-muted-foreground text-sm', className)}
      {...props}
    />
  )
}
