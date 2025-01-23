// import { useAPI } from '../../lib/api-context'
// import { createCollectionOptions } from '@colette/query'
// import { FormMessage } from '@colette/react-ui/components/form'
// import { Button } from '@colette/react-ui/components/ui/button'
// import {
//   Dialog,
//   DialogContent,
//   DialogDescription,
//   DialogFooter,
//   DialogHeader,
//   DialogTitle,
//   DialogTrigger,
// } from '@colette/react-ui/components/ui/dialog'
// import { Input } from '@colette/react-ui/components/ui/input'
// import { Label } from '@colette/react-ui/components/ui/label'
// import { SidebarGroupAction } from '@colette/react-ui/components/ui/sidebar'
// import { useForm } from '@tanstack/react-form'
// import { useMutation, useQueryClient } from '@tanstack/react-query'
// import { Plus } from 'lucide-react'
// import { type FC, useState } from 'react'
// import { useLocation } from 'wouter'
// import { z } from 'zod'

// export const CreateCollectionModal: FC = () => {
//   const api = useAPI()
//   const [, navigate] = useLocation()
//   const queryClient = useQueryClient()

//   const [isOpen, setOpen] = useState(false)

//   const form = useForm({
//     defaultValues: {
//       title: '',
//     },
//     onSubmit: ({ value }) => mutation.mutate(value),
//   })

//   const mutation = useMutation(
//     createCollectionOptions(api, queryClient, {
//       onSuccess: (collection) => {
//         form.reset()
//         setOpen(false)

//         navigate(`/collections/${collection.id}`)
//       },
//     }),
//   )

//   return (
//     <Dialog open={isOpen} onOpenChange={setOpen}>
//       <DialogTrigger asChild>
//         <SidebarGroupAction>
//           <Plus />
//         </SidebarGroupAction>
//       </DialogTrigger>
//       <DialogContent className="gap-6">
//         <DialogHeader>
//           <DialogTitle>Add Collection</DialogTitle>
//           <DialogDescription>
//             Create a new collection of bookmarks.
//           </DialogDescription>
//         </DialogHeader>
//         <form
//           onSubmit={(e) => {
//             e.preventDefault()
//             form.handleSubmit()
//           }}
//         >
//           <form.Field
//             name="title"
//             validators={{
//               onSubmit: z.string().min(1, 'Title cannot be empty'),
//             }}
//           >
//             {(field) => (
//               <div className="space-y-1">
//                 <Label>Title</Label>
//                 <Input
//                   value={field.state.value}
//                   onChange={(ev) => field.handleChange(ev.target.value)}
//                 />
//                 <FormMessage>
//                   {field.state.meta.errors[0]?.toString()}
//                 </FormMessage>
//               </div>
//             )}
//           </form.Field>
//           <DialogFooter className="mt-6">
//             <Button type="submit" disabled={mutation.isPending}>
//               Submit
//             </Button>
//           </DialogFooter>
//         </form>
//       </DialogContent>
//     </Dialog>
//   )
// }
