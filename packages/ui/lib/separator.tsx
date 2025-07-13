import { cn } from './utils'
import { type HTMLArkProps, ark } from '@ark-ui/react'

export type SeparatorProps = HTMLArkProps<'div'> & {
  orientation?: 'vertical' | 'horizontal'
  decorative?: boolean
}

export const Separator = ({
  className,
  orientation = 'horizontal',
  decorative = true,
  ...props
}: SeparatorProps) => {
  const semanticProps = decorative
    ? { role: 'none' }
    : {
        'aria-orientation':
          orientation === 'vertical' ? orientation : undefined,
        role: 'separator',
      }

  return (
    <ark.div
      data-scope="separator"
      data-orientation={orientation}
      className={cn(
        'bg-border shrink-0 data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px',
        className,
      )}
      {...semanticProps}
      {...props}
    />
  )
}
