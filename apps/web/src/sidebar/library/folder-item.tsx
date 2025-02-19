import { FolderContents } from './folder-contents'
import { Folder as AppFolder } from '@colette/core'
import { ChevronRight, Folder, MoreHorizontal } from 'lucide-react'
import { FC } from 'react'
import { SidebarMenuSubAction } from '~/components/sidebar-menu-sub-action'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '~/components/ui/collapsible'
import { Dialog, DialogTrigger } from '~/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '~/components/ui/sidebar'

export const FolderItem: FC<{ folder: AppFolder }> = ({ folder }) => {
  return (
    <SidebarMenuSubItem>
      <Collapsible className="group/collapsible [&[data-state=open]>a>svg:first-child]:rotate-90">
        <CollapsibleTrigger asChild>
          <SidebarMenuSubButton>
            <ChevronRight className="transition-transform" />
            <Folder />
            {folder.title}
          </SidebarMenuSubButton>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub className="mr-0 ml-3.5 pr-0 pl-2.5">
            <FolderContents folderId={folder.id} />
          </SidebarMenuSub>
        </CollapsibleContent>
      </Collapsible>
      <Dialog>
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger asChild>
            <SidebarMenuSubAction showOnHover>
              <MoreHorizontal />
            </SidebarMenuSubAction>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="right">
            <DialogTrigger asChild>
              <DropdownMenuItem>New Folder</DropdownMenuItem>
            </DialogTrigger>
          </DropdownMenuContent>
        </DropdownMenu>
      </Dialog>
    </SidebarMenuSubItem>
  )
}
