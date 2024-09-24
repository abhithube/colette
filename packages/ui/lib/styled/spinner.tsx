import { ark } from '@ark-ui/react/factory'
import { styled } from '@colette/styled-system/jsx'
import { spinner } from '@colette/styled-system/recipes'
import type { ComponentProps } from '@colette/styled-system/types'

export type SpinnerProps = ComponentProps<typeof Spinner>
export const Spinner = styled(ark.div, spinner)
