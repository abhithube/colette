import { cn } from './utils'
import { ark, HTMLArkProps } from '@ark-ui/react'

export type RootProps = HTMLArkProps<'div'>

export const Root = ({ className, ...props }: RootProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="root"
      className={cn(
        'bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm',
        className,
      )}
      {...props}
    />
  )
}

export type HeaderProps = HTMLArkProps<'div'>

export const Header = ({ className, ...props }: HeaderProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="header"
      className={cn(
        '@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-1.5 px-6 has-data-[scope=card]:has-data-[part=action]:grid-cols-[1fr_auto] [.border-b]:pb-6',
        className,
      )}
      {...props}
    />
  )
}

export type TitleProps = HTMLArkProps<'div'>

export const Title = ({ className, ...props }: TitleProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="title"
      className={cn('leading-none font-semibold', className)}
      {...props}
    />
  )
}

export type DescriptionProps = HTMLArkProps<'div'>

export const Description = ({ className, ...props }: DescriptionProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="description"
      className={cn('text-muted-foreground text-sm', className)}
      {...props}
    />
  )
}

export type ActionProps = HTMLArkProps<'div'>

export const Action = ({ className, ...props }: ActionProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="action"
      className={cn(
        'col-start-2 row-span-2 row-start-1 self-start justify-self-end',
        className,
      )}
      {...props}
    />
  )
}

export type ContentProps = HTMLArkProps<'div'>

export const Content = ({ className, ...props }: ContentProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="content"
      className={cn('px-6', className)}
      {...props}
    />
  )
}

export type FooterProps = HTMLArkProps<'div'>

export const Footer = ({ className, ...props }: FooterProps) => {
  return (
    <ark.div
      data-scope="card"
      data-part="footer"
      className={cn('flex items-center px-6 [.border-t]:pt-6', className)}
      {...props}
    />
  )
}
