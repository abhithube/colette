import { ark } from '@ark-ui/react/factory'
import { styled } from '@colette/styled-system/jsx'
import { badge } from '@colette/styled-system/recipes'
import type { ComponentProps } from '@colette/styled-system/types'

export type BadgeProps = ComponentProps<typeof Badge>
export const Badge = styled(ark.div, badge)
