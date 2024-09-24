import { ark } from '@ark-ui/react/factory'
import { styled } from '@colette/styled-system/jsx'
import { link } from '@colette/styled-system/recipes'
import type { ComponentProps } from '@colette/styled-system/types'

export type LinkProps = ComponentProps<typeof Link>
export const Link = styled(ark.a, link)
