import { Dialog } from '@ark-ui/solid'
import { type VariantProps, cva } from 'class-variance-authority'
import X from 'lucide-solid/icons/x'
import {
  type Component,
  type ComponentProps,
  type ParentComponent,
  splitProps,
} from 'solid-js'
import { Portal } from 'solid-js/web'
import { cn } from '~/lib/utils'

export const Sheet: Component<Dialog.RootProps> = (props) => {
  return <Dialog.Root {...props} />
}

export const SheetTrigger: Component<Dialog.TriggerProps> = (props) => {
  return <Dialog.Trigger {...props} />
}

export const SheetClose: Component<Dialog.CloseTriggerProps> = (props) => {
  return <Dialog.CloseTrigger {...props} />
}

const portalVariants = cva('fixed inset-0 z-50 flex', {
  variants: {
    position: {
      top: 'items-start',
      bottom: 'items-end',
      left: 'justify-start',
      right: 'justify-end',
    },
  },
  defaultVariants: { position: 'right' },
})

export const SheetPortal: ParentComponent<
  VariantProps<typeof portalVariants>
> = (props) => {
  const [local, others] = splitProps(props, ['position', 'children'])

  return (
    <Portal {...others}>
      <div class={portalVariants({ position: local.position })}>
        {local.children}
      </div>
    </Portal>
  )
}

export const SheetOverlay: Component<Dialog.BackdropProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Dialog.Backdrop
      class={cn(
        'data-[closed=]:fade-out-0 data-[expanded=]:fade-in-0 fixed inset-0 z-50 bg-black/80 data-[closed=]:animate-out data-[expanded=]:animate-in',
        local.class,
      )}
      {...others}
    />
  )
}

const sheetVariants = cva(
  'fixed z-50 gap-4 bg-background p-6 shadow-lg transition ease-in-out data-[closed=]:animate-out data-[expanded=]:animate-in data-[closed=]:duration-300 data-[expanded=]:duration-500',
  {
    variants: {
      position: {
        top: 'data-[closed=]:slide-out-to-top data-[expanded=]:slide-in-from-top inset-x-0 top-0 border-b',
        bottom:
          'data-[closed=]:slide-out-to-bottom data-[expanded=]:slide-in-from-bottom inset-x-0 bottom-0 border-t',
        left: 'data-[closed=]:slide-out-to-left data-[expanded]:slide-in-from-left inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm',
        right:
          'data-[closed=]:slide-out-to-right data-[expanded=]:slide-in-from-right inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm',
      },
    },
    defaultVariants: {
      position: 'right',
    },
  },
)

export const SheetContent: ParentComponent<
  Dialog.ContentProps & VariantProps<typeof sheetVariants>
> = (props) => {
  const [local, others] = splitProps(props, ['class', 'position', 'children'])

  return (
    <SheetPortal position={local.position}>
      <SheetOverlay />
      <Dialog.Content
        class={cn(
          sheetVariants({ position: local.position }),
          local.class,
          'max-h-screen overflow-y-auto',
        )}
        {...others}
      >
        {local.children}
        <Dialog.CloseTrigger class="absolute top-4 right-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary">
          <X class="h-4 w-4" />
          <span class="sr-only">Close</span>
        </Dialog.CloseTrigger>
      </Dialog.Content>
    </SheetPortal>
  )
}

export const SheetHeader: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      class={cn(
        'flex flex-col space-y-2 text-center sm:text-left',
        local.class,
      )}
      {...others}
    />
  )
}

export const SheetFooter: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      class={cn(
        'flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2',
        local.class,
      )}
      {...others}
    />
  )
}

export const SheetTitle: Component<Dialog.TitleProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Dialog.Title
      class={cn('font-semibold text-foreground text-lg', local.class)}
      {...others}
    />
  )
}

export const SheetDescription: Component<Dialog.DescriptionProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Dialog.Description
      class={cn('text-muted-foreground text-sm', local.class)}
      {...others}
    />
  )
}
