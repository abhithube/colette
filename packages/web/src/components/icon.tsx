import { cn } from '@/lib/utils'
import { type VariantProps, cva } from 'class-variance-authority'
import * as React from 'react'

const iconVariants = cva('shrink-0', {
	variants: {
		size: {
			default: 'h-4 w-4',
			lg: 'h-5 w-5',
		},
	},
	defaultVariants: {
		size: 'default',
	},
})

export interface IconProps
	extends React.SVGAttributes<SVGSVGElement>,
		VariantProps<typeof iconVariants> {
	value: React.ExoticComponent<
		React.PropsWithoutRef<React.SVGAttributes<SVGSVGElement>> &
			React.RefAttributes<SVGSVGElement>
	>
}

const Icon = React.forwardRef<SVGSVGElement, IconProps>(
	({ className, size, value, ...props }, ref) => {
		return React.createElement(value, {
			className: cn(iconVariants({ size, className })),
			ref,
			...props,
		})
	},
)
Icon.displayName = 'Icon'

export { Icon, iconVariants }
