import { Bookmark, BookmarkScraped, Tag } from '@colette/core'
import { formOptions } from '@tanstack/react-form'
import { z } from 'zod'

export const SCRAPE_BOOKMARK_FORM = 'scrape-bookmark'

export const scrapeBookmarkFormOptions = () =>
  formOptions({
    defaultValues: {
      url: '',
    },
    validators: {
      onBlur: z.object({
        url: z.string().url('URL is not valid'),
      }),
    },
  })

export const CREATE_BOOKMARK_FORM = 'create-bookmark'

export const createBookmarkFormOptions = (initial: BookmarkScraped) =>
  formOptions({
    defaultValues: {
      title: initial.title,
      thumbnailUrl: initial.thumbnailUrl,
      publishedAt: initial.publishedAt,
      author: initial.author,
    },
    validators: {
      onBlur: z.object({
        title: z.string().min(1, "Title can't be empty"),
        thumbnailUrl: z.string().url('URL is not valid').nullable(),
        publishedAt: z.string().datetime('Date is not valid').nullable(),
        author: z.string().min(1, 'Author cannot be empty').nullable(),
      }),
    },
  })

export const UPDATE_BOOKMARK_FORM = 'update-bookmark'

export const updateBookmarkFormOptions = (initial: Bookmark) =>
  formOptions({
    defaultValues: {
      title: initial.title,
      thumbnailUrl: initial.thumbnailUrl,
      publishedAt: initial.publishedAt,
      author: initial.author,
    },
    validators: {
      onBlur: z.object({
        title: z.string().min(1, "Title can't be empty"),
        thumbnailUrl: z.string().url('URL is not valid').nullable(),
        publishedAt: z.string().datetime('Date is not valid').nullable(),
        author: z.string().min(1, 'Author cannot be empty').nullable(),
      }),
    },
  })

export const LINK_BOOKMARK_TAGS_FORM = 'link-bookmark-tags'

export const linkBookmarkTagsFormOptions = (initial?: Tag[]) =>
  formOptions({
    defaultValues: {
      tagIds: initial?.map((tag) => tag.id) ?? [],
    },
    validators: {
      onBlur: z.object({
        tagIds: z.array(z.string().uuid('UUID is not valid')),
      }),
    },
  })
