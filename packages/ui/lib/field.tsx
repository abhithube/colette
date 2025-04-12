import { cn } from './utils'
import { Field } from '@ark-ui/react'

export type RootProps = Field.RootProps

export const Root = Field.Root

export type InputProps = Field.InputProps

export const Input = ({ className, ...props }: InputProps) => {
  return (
    <Field.Input
      className={cn(
        'file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
        'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
        'aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive',
        className,
      )}
      {...props}
    />
  )
}

export type LabelProps = Field.LabelProps

export const Label = ({ className, ...props }: LabelProps) => {
  return (
    <Field.Label
      className={cn(
        'flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50',
        className,
      )}
      {...props}
    />
  )
}

export type ErrorTextProps = Field.ErrorTextProps

export const ErrorText = ({
  className,
  children,
  ...props
}: ErrorTextProps) => {
  if (!children) return null

  return (
    <Field.ErrorText
      className={cn('text-destructive text-sm', className)}
      {...props}
    >
      {children}
    </Field.ErrorText>
  )
}

export type HelperTextProps = Field.HelperTextProps

export const HelperText = ({ className, ...props }: HelperTextProps) => {
  return (
    <Field.HelperText
      className={cn('text-muted-foreground text-sm', className)}
      {...props}
    />
  )
}
