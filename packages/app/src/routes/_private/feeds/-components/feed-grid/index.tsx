import type { Feed } from '@colette/core'
import { Badge, HStack, Link, Table, Text } from '@colette/ui'
import { Link as TLink } from '@tanstack/react-router'
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table'
import { FeedRowActions } from './actions'

const columnHelper = createColumnHelper<Feed>()

const columns = [
  columnHelper.accessor(
    (row) => ({
      id: row.id,
      title: row.title ?? row.originalTitle,
      tags: row.tags,
    }),
    {
      id: 'title',
      header: 'Title',
      cell: (props) => (
        <HStack>
          <Link asChild>
            <TLink to="/feeds/$id" params={{ id: props.getValue().id }}>
              <Text as="span" truncate title={props.getValue().title}>
                {props.getValue().title}
              </Text>
            </TLink>
          </Link>
          <HStack>
            {props.getValue().tags?.map((tag) => (
              <Badge key={tag.id}>{tag.title}</Badge>
            ))}
          </HStack>
        </HStack>
      ),
    },
  ),
  columnHelper.display({
    id: 'actions',
    header: 'Actions',
    cell: (props) => <FeedRowActions feed={props.row.original} />,
  }),
]

type Props = {
  feeds: Feed[]
}

export function FeedGrid({ feeds }: Props) {
  const table = useReactTable({
    data: feeds,
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  return (
    <Table.Root>
      <Table.Head>
        {table.getHeaderGroups().map((headerGroup) => (
          <Table.Row key={headerGroup.id}>
            {headerGroup.headers.map((header) => (
              <Table.Header key={header.id}>
                {flexRender(
                  header.column.columnDef.header,
                  header.getContext(),
                )}
              </Table.Header>
            ))}
          </Table.Row>
        ))}
      </Table.Head>
      <Table.Body>
        {table.getRowModel().rows.map((row) => (
          <Table.Row key={row.id}>
            {row.getVisibleCells().map((cell) => (
              <Table.Cell key={cell.id}>
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </Table.Cell>
            ))}
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  )
}
