import { ark } from '@ark-ui/react/factory'
import { styled } from '@colette/styled-system/jsx'
import { button } from '@colette/styled-system/recipes'
import type { ComponentProps } from '@colette/styled-system/types'

export type ButtonProps = ComponentProps<typeof Button>
export const Button = styled(ark.button, button)
