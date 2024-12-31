import { type DialogRootProps, type HTMLArkProps, ark } from '@ark-ui/solid'
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'
import PanelLeft from 'lucide-solid/icons/panel-left'
import {
  type Accessor,
  type Component,
  type ComponentProps,
  type JSX,
  Match,
  type ParentComponent,
  Show,
  Switch,
  createContext,
  createEffect,
  createSignal,
  mergeProps,
  onCleanup,
  splitProps,
  useContext,
} from 'solid-js'
import { useIsMobile } from '~/lib/use-mobile'
import { cn } from '~/lib/utils'
import { Button, type ButtonProps } from './button'
import { Separator, type SeparatorProps } from './separator'
import { Sheet, SheetContent } from './sheet'
import { Tooltip, TooltipContent, TooltipTrigger } from './tooltip'

const SIDEBAR_COOKIE_NAME = 'sidebar:state'
const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7
const SIDEBAR_WIDTH = '16rem'
const SIDEBAR_WIDTH_MOBILE = '18rem'
const SIDEBAR_WIDTH_ICON = '3rem'
const SIDEBAR_KEYBOARD_SHORTCUT = 'b'

type SidebarContext = {
  state: Accessor<'expanded' | 'collapsed'>
  open: Accessor<boolean>
  setOpen: (open: boolean) => void
  openMobile: Accessor<boolean>
  setOpenMobile: (open: boolean) => void
  isMobile: Accessor<boolean>
  toggleSidebar: () => void
}

const SidebarContext = createContext<SidebarContext | null>(null)

const useSidebar = () => {
  const context = useContext(SidebarContext)
  if (!context) {
    throw new Error('useSidebar must be used within a SidebarProvider.')
  }

  return context
}

type SidebarProviderProps = Omit<ComponentProps<'div'>, 'style'> & {
  defaultOpen?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
  style?: JSX.CSSProperties
}

export const SidebarProvider: ParentComponent<SidebarProviderProps> = (
  rawProps,
) => {
  const props = mergeProps({ defaultOpen: true }, rawProps)
  const [local, others] = splitProps(props, [
    'defaultOpen',
    'open',
    'onOpenChange',
    'class',
    'style',
    'children',
  ])

  const isMobile = useIsMobile()
  const [openMobile, setOpenMobile] = createSignal(false)

  const [_open, _setOpen] = createSignal(local.defaultOpen)
  const open = () => local.open ?? _open()
  const setOpen = (value: boolean | ((value: boolean) => boolean)) => {
    const openState = typeof value === 'function' ? value(open()) : value
    if (local.onOpenChange) {
      local.onOpenChange(openState)
    } else {
      _setOpen(openState)
    }

    document.cookie = `${SIDEBAR_COOKIE_NAME}=${open()}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`
  }

  const toggleSidebar = () => {
    return isMobile()
      ? setOpenMobile((open) => !open)
      : setOpen((open) => !open)
  }

  createEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === SIDEBAR_KEYBOARD_SHORTCUT &&
        (event.metaKey || event.ctrlKey)
      ) {
        event.preventDefault()
        toggleSidebar()
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    onCleanup(() => window.removeEventListener('keydown', handleKeyDown))
  })

  const state = () => (open() ? 'expanded' : 'collapsed')

  return (
    <SidebarContext.Provider
      value={{
        state,
        open,
        setOpen,
        isMobile,
        openMobile,
        setOpenMobile,
        toggleSidebar,
      }}
    >
      <div
        style={{
          '--sidebar-width': SIDEBAR_WIDTH,
          '--sidebar-width-icon': SIDEBAR_WIDTH_ICON,
          ...local.style,
        }}
        class={cn(
          'group/sidebar-wrapper flex min-h-svh w-full text-sidebar-foreground has-[[data-variant=inset]]:bg-sidebar',
          local.class,
        )}
        {...others}
      >
        {local.children}
      </div>
    </SidebarContext.Provider>
  )
}

type SidebarProps = ComponentProps<'div'> & {
  side?: 'left' | 'right'
  variant?: 'sidebar' | 'floating' | 'inset'
  collapsible?: 'offcanvas' | 'icon' | 'none'
}

export const Sidebar: ParentComponent<SidebarProps> = (rawProps) => {
  const props = mergeProps(
    {
      side: 'left',
      variant: 'sidebar',
      collapsible: 'offcanvas',
    } as SidebarProps,
    rawProps,
  )
  const [local, others] = splitProps(props, [
    'side',
    'variant',
    'collapsible',
    'class',
    'children',
  ])

  const { isMobile, state, openMobile, setOpenMobile } = useSidebar()

  return (
    <Switch>
      <Match when={local.collapsible === 'none'}>
        <div
          class={cn(
            'flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground',
            local.class,
          )}
          {...others}
        >
          {local.children}
        </div>
      </Match>
      <Match when={isMobile()}>
        <Sheet
          open={openMobile()}
          onOpenChange={(details) => setOpenMobile(details.open)}
          {...(others as DialogRootProps)}
        >
          <SheetContent
            data-sidebar="sidebar"
            data-mobile="true"
            class="w-(--sidebar-width) bg-sidebar p-0 text-sidebar-foreground [&>button]:hidden"
            style={{
              '--sidebar-width': SIDEBAR_WIDTH_MOBILE,
            }}
            position={local.side}
          >
            <div class="flex size-full flex-col">{local.children}</div>
          </SheetContent>
        </Sheet>
      </Match>
      <Match when={!isMobile()}>
        <div
          class="group peer hidden md:block"
          data-state={state()}
          data-collapsible={state() === 'collapsed' ? local.collapsible : ''}
          data-variant={local.variant}
          data-side={local.side}
        >
          <div
            class={cn(
              'relative h-svh w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear',
              'group-data-[collapsible=offcanvas]:w-0',
              'group-data-[side=right]:rotate-180',
              local.variant === 'floating' || local.variant === 'inset'
                ? 'group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)_+_theme(spacing.4))]'
                : 'group-data-[collapsible=icon]:w-(--sidebar-width-icon)',
            )}
          />
          <div
            class={cn(
              'fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear md:flex',
              local.side === 'left'
                ? 'left-0 group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)]'
                : 'right-0 group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)]',
              local.variant === 'floating' || local.variant === 'inset'
                ? 'p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)_+_theme(spacing.4)_+2px)]'
                : 'group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l',
              local.class,
            )}
            {...others}
          >
            <div
              data-sidebar="sidebar"
              class="flex size-full flex-col bg-sidebar group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:border-sidebar-border group-data-[variant=floating]:shadow"
            >
              {local.children}
            </div>
          </div>
        </div>
      </Match>
    </Switch>
  )
}

export const SidebarTrigger: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'onClick'])
  const { toggleSidebar } = useSidebar()

  return (
    <Button
      data-sidebar="trigger"
      variant="ghost"
      size="icon"
      class={cn('h-7 w-7', local.class)}
      onClick={(event) => {
        if (typeof local.onClick === 'function') {
          local.onClick(event)
        }

        toggleSidebar()
      }}
      {...others}
    >
      <PanelLeft />
      <span class="sr-only">Toggle Sidebar</span>
    </Button>
  )
}

export const SidebarRail: Component<ComponentProps<'button'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  const { toggleSidebar } = useSidebar()

  return (
    <button
      data-sidebar="rail"
      aria-label="Toggle Sidebar"
      tabIndex={-1}
      onClick={toggleSidebar}
      title="Toggle Sidebar"
      class={cn(
        '-translate-x-1/2 group-data-[side=left]:-right-4 absolute inset-y-0 z-20 hidden w-4 transition-all ease-linear after:absolute after:inset-y-0 after:left-1/2 after:w-[2px] hover:after:bg-sidebar-border group-data-[side=right]:left-0 sm:flex',
        '[[data-side=left]_&]:cursor-w-resize [[data-side=right]_&]:cursor-e-resize',
        '[[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize',
        'group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:hover:bg-sidebar group-data-[collapsible=offcanvas]:after:left-full',
        '[[data-side=left][data-collapsible=offcanvas]_&]:-right-2',
        '[[data-side=right][data-collapsible=offcanvas]_&]:-left-2',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarInset: Component<ComponentProps<'main'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <main
      class={cn(
        'relative flex min-h-svh flex-1 flex-col bg-background',
        'peer-data-[variant=inset]:min-h-[calc(100svh-theme(spacing.4))] md:peer-data-[state=collapsed]:peer-data-[variant=inset]:ml-2 md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:ml-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarHeader: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="header"
      class={cn('flex flex-col gap-2 p-2', local.class)}
      {...others}
    />
  )
}

export const SidebarFooter: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="footer"
      class={cn('flex flex-col gap-2 p-2', local.class)}
      {...others}
    />
  )
}

export const SidebarSeparator: Component<SeparatorProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Separator
      data-sidebar="separator"
      class={cn('mx-2 w-auto bg-sidebar-border', local.class)}
      {...others}
    />
  )
}

export const SidebarContent: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="content"
      class={cn(
        'flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=icon]:overflow-hidden',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarGroup: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="group"
      class={cn('relative flex w-full min-w-0 flex-col p-2', local.class)}
      {...others}
    />
  )
}

export const SidebarGroupLabel: Component<HTMLArkProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <ark.div
      data-sidebar="group-label"
      class={cn(
        'flex h-8 shrink-0 items-center rounded-md px-2 font-medium text-sidebar-foreground/70 text-xs outline-none ring-sidebar-ring transition-[margin,opa] duration-200 ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0',
        'group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarGroupAction: Component<HTMLArkProps<'button'>> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <ark.button
      class={cn(
        'absolute top-3.5 right-3 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground outline-none ring-sidebar-ring transition-transform hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0',
        'after:-inset-2 after:absolute after:md:hidden',
        'group-data-[collapsible=icon]:hidden',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarGroupContent: Component<ComponentProps<'div'>> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="group-content"
      class={cn('w-full text-sm', local.class)}
      {...others}
    />
  )
}

export const SidebarMenu: Component<ComponentProps<'ul'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <ul
      data-sidebar="menu"
      class={cn('flex w-full min-w-0 flex-col gap-1', local.class)}
      {...others}
    />
  )
}

export const SidebarMenuItem: Component<ComponentProps<'li'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <li
      data-sidebar="menu-item"
      class={cn('group/menu-item relative', local.class)}
      {...others}
    />
  )
}

const sidebarMenuButtonVariants = cva(
  'peer/menu-button group-data-[collapsible=icon]:!size-8 group-data-[collapsible=icon]:!p-2 flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm outline-none ring-sidebar-ring transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-has-[[data-sidebar=menu-action]]/menu-item:pr-8 aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[active=true]:bg-sidebar-accent data-[active=true]:font-medium data-[active=true]:text-sidebar-accent-foreground data-[state=open]:hover:bg-sidebar-accent data-[state=open]:hover:text-sidebar-accent-foreground [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
        outline:
          'bg-background shadow-[0_0_0_1px_hsl(var(--sidebar-border))] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground hover:shadow-[0_0_0_1px_hsl(var(--sidebar-accent))]',
      },
      size: {
        default: 'h-8 text-sm',
        sm: 'h-7 text-xs',
        lg: 'group-data-[collapsible=icon]:!p-0 h-12 text-sm',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

type SidebarMenuButtonProps = HTMLArkProps<'button'> &
  VariantProps<typeof sidebarMenuButtonVariants> & {
    isActive?: boolean
    tooltip?: string
  }

export const SidebarMenuButton: Component<SidebarMenuButtonProps> = (
  rawProps,
) => {
  const props = mergeProps(
    {
      isActive: false,
      variant: 'default',
      size: 'default',
    } as SidebarMenuButtonProps,
    rawProps,
  )
  const [local, others] = splitProps(props, [
    'isActive',
    'tooltip',
    'variant',
    'size',
    'class',
  ])

  const { isMobile, state } = useSidebar()

  const button = (
    <ark.button
      class={cn(
        sidebarMenuButtonVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      {...others}
    />
  )

  return (
    <Show when={local.tooltip} fallback={button}>
      <Tooltip positioning={{ placement: 'right' }}>
        <TooltipTrigger class="w-full">{button}</TooltipTrigger>
        <TooltipContent hidden={state() !== 'collapsed' || isMobile()}>
          {local.tooltip}
        </TooltipContent>
      </Tooltip>
    </Show>
  )
}

export const SidebarMenuAction: Component<
  HTMLArkProps<'button'> & { showOnHover?: boolean }
> = (rawProps) => {
  const props = mergeProps({ showOnHover: false }, rawProps)
  const [local, others] = splitProps(props, ['class', 'showOnHover'])

  return (
    <ark.button
      data-sidebar="menu-action"
      class={cn(
        'absolute top-1.5 right-1 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground outline-none ring-sidebar-ring transition-transform hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 peer-hover/menu-button:text-sidebar-accent-foreground [&>svg]:size-4 [&>svg]:shrink-0',
        'after:-inset-2 after:absolute after:md:hidden',
        'peer-data-[size=sm]/menu-button:top-1',
        'peer-data-[size=default]/menu-button:top-1.5',
        'peer-data-[size=lg]/menu-button:top-2.5',
        'group-data-[collapsible=icon]:hidden',
        local.showOnHover &&
          'group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 data-[state=open]:opacity-100 peer-data-[active=true]/menu-button:text-sidebar-accent-foreground md:opacity-0',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarMenuBadge: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      data-sidebar="menu-badge"
      class={cn(
        'pointer-events-none absolute right-1 flex h-5 min-w-5 select-none items-center justify-center rounded-md px-1 font-medium text-sidebar-foreground text-xs tabular-nums',
        'peer-hover/menu-button:text-sidebar-accent-foreground peer-data-[active=true]/menu-button:text-sidebar-accent-foreground',
        'peer-data-[size=sm]/menu-button:top-1',
        'peer-data-[size=default]/menu-button:top-1.5',
        'peer-data-[size=lg]/menu-button:top-2.5',
        'group-data-[collapsible=icon]:hidden',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarMenuSub: Component<ComponentProps<'ul'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <ul
      data-sidebar="menu-sub"
      class={cn(
        'mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-sidebar-border border-l px-2.5 py-0.5',
        'group-data-[collapsible=icon]:hidden',
        local.class,
      )}
      {...others}
    />
  )
}

export const SidebarMenuSubItem: Component<ComponentProps<'li'>> = (props) => (
  <li {...props} />
)

type SidebarMenuSubButtonProps = HTMLArkProps<'a'> & {
  size?: 'sm' | 'md'
  isActive?: boolean
}

export const SidebarMenuSubButton: Component<SidebarMenuSubButtonProps> = (
  rawProps,
) => {
  const props = mergeProps(
    { size: 'md' } as SidebarMenuSubButtonProps,
    rawProps,
  )
  const [local, others] = splitProps(props, ['class', 'size', 'isActive'])

  return (
    <ark.a
      data-sidebar="menu-sub-button"
      data-size={local.size}
      data-active={local.isActive}
      class={cn(
        '-translate-x-px flex h-7 min-w-0 items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground outline-none ring-sidebar-ring hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-sidebar-accent-foreground',
        'data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground',
        local.size === 'sm' && 'text-xs',
        local.size === 'md' && 'text-sm',
        'group-data-[collapsible=icon]:hidden',
        local.class,
      )}
      {...others}
    />
  )
}
