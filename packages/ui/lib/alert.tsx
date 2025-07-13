import { cn } from './utils'
import { ark, type HTMLArkProps } from '@ark-ui/react'
import { cva, type VariantProps } from 'class-variance-authority'

export const alertVariants = cva(
  'relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] grid-cols-[0_1fr] has-[>svg]:gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current',
  {
    variants: {
      variant: {
        default: 'bg-card text-card-foreground',
        destructive:
          'text-destructive bg-card [&>svg]:text-current *:data-[scope=alert]:*:data-[part=description]:text-destructive/90',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export type RootProps = HTMLArkProps<'div'> & VariantProps<typeof alertVariants>

export const Root = ({ className, variant, ...props }: RootProps) => {
  return (
    <ark.div
      role="alert"
      data-scope="alert"
      data-part="root"
      className={cn(alertVariants({ variant }), className)}
      {...props}
    />
  )
}

export type TitleProps = HTMLArkProps<'h5'>

export const Title = ({ className, ...props }: TitleProps) => {
  return (
    <ark.h5
      data-scope="alert"
      data-part="title"
      className={cn(
        'col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight',
        className,
      )}
      {...props}
    />
  )
}

export type DescriptionProps = HTMLArkProps<'p'>

export const Description = ({ className, ...props }: DescriptionProps) => {
  return (
    <ark.p
      data-scope="alert"
      data-part="description"
      className={cn(
        'text-muted-foreground col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed',
        className,
      )}
      {...props}
    />
  )
}
