import { ark } from '@ark-ui/react/factory'
import { styled } from '@colette/styled-system/jsx'
import { icon } from '@colette/styled-system/recipes'
import type { ComponentProps } from '@colette/styled-system/types'

export type IconProps = ComponentProps<typeof Icon>
export const Icon = styled(ark.svg, icon, {
  defaultProps: { asChild: true },
})
