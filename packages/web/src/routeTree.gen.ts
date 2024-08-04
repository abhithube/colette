/* prettier-ignore-start */

/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file is auto-generated by TanStack Router

// Import Routes

import { Route as rootRoute } from './routes/__root'
import { Route as LoginImport } from './routes/login'
import { Route as PrivateImport } from './routes/_private'
import { Route as PrivateIndexImport } from './routes/_private/index'
import { Route as PrivateFeedsImport } from './routes/_private/feeds'
import { Route as PrivateBookmarksImport } from './routes/_private/bookmarks'
import { Route as PrivateFeedsIndexImport } from './routes/_private/feeds/index'
import { Route as PrivateBookmarksIndexImport } from './routes/_private/bookmarks/index'
import { Route as PrivateFeedsArchivedImport } from './routes/_private/feeds/archived'
import { Route as PrivateFeedsIdImport } from './routes/_private/feeds/$id'
import { Route as PrivateBookmarksStashImport } from './routes/_private/bookmarks/stash'

// Create/Update Routes

const LoginRoute = LoginImport.update({
  path: '/login',
  getParentRoute: () => rootRoute,
} as any)

const PrivateRoute = PrivateImport.update({
  id: '/_private',
  getParentRoute: () => rootRoute,
} as any)

const PrivateIndexRoute = PrivateIndexImport.update({
  path: '/',
  getParentRoute: () => PrivateRoute,
} as any)

const PrivateFeedsRoute = PrivateFeedsImport.update({
  path: '/feeds',
  getParentRoute: () => PrivateRoute,
} as any)

const PrivateBookmarksRoute = PrivateBookmarksImport.update({
  path: '/bookmarks',
  getParentRoute: () => PrivateRoute,
} as any)

const PrivateFeedsIndexRoute = PrivateFeedsIndexImport.update({
  path: '/',
  getParentRoute: () => PrivateFeedsRoute,
} as any)

const PrivateBookmarksIndexRoute = PrivateBookmarksIndexImport.update({
  path: '/',
  getParentRoute: () => PrivateBookmarksRoute,
} as any)

const PrivateFeedsArchivedRoute = PrivateFeedsArchivedImport.update({
  path: '/archived',
  getParentRoute: () => PrivateFeedsRoute,
} as any)

const PrivateFeedsIdRoute = PrivateFeedsIdImport.update({
  path: '/$id',
  getParentRoute: () => PrivateFeedsRoute,
} as any)

const PrivateBookmarksStashRoute = PrivateBookmarksStashImport.update({
  path: '/stash',
  getParentRoute: () => PrivateBookmarksRoute,
} as any)

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/_private': {
      id: '/_private'
      path: ''
      fullPath: ''
      preLoaderRoute: typeof PrivateImport
      parentRoute: typeof rootRoute
    }
    '/login': {
      id: '/login'
      path: '/login'
      fullPath: '/login'
      preLoaderRoute: typeof LoginImport
      parentRoute: typeof rootRoute
    }
    '/_private/bookmarks': {
      id: '/_private/bookmarks'
      path: '/bookmarks'
      fullPath: '/bookmarks'
      preLoaderRoute: typeof PrivateBookmarksImport
      parentRoute: typeof PrivateImport
    }
    '/_private/feeds': {
      id: '/_private/feeds'
      path: '/feeds'
      fullPath: '/feeds'
      preLoaderRoute: typeof PrivateFeedsImport
      parentRoute: typeof PrivateImport
    }
    '/_private/': {
      id: '/_private/'
      path: '/'
      fullPath: '/'
      preLoaderRoute: typeof PrivateIndexImport
      parentRoute: typeof PrivateImport
    }
    '/_private/bookmarks/stash': {
      id: '/_private/bookmarks/stash'
      path: '/stash'
      fullPath: '/bookmarks/stash'
      preLoaderRoute: typeof PrivateBookmarksStashImport
      parentRoute: typeof PrivateBookmarksImport
    }
    '/_private/feeds/$id': {
      id: '/_private/feeds/$id'
      path: '/$id'
      fullPath: '/feeds/$id'
      preLoaderRoute: typeof PrivateFeedsIdImport
      parentRoute: typeof PrivateFeedsImport
    }
    '/_private/feeds/archived': {
      id: '/_private/feeds/archived'
      path: '/archived'
      fullPath: '/feeds/archived'
      preLoaderRoute: typeof PrivateFeedsArchivedImport
      parentRoute: typeof PrivateFeedsImport
    }
    '/_private/bookmarks/': {
      id: '/_private/bookmarks/'
      path: '/'
      fullPath: '/bookmarks/'
      preLoaderRoute: typeof PrivateBookmarksIndexImport
      parentRoute: typeof PrivateBookmarksImport
    }
    '/_private/feeds/': {
      id: '/_private/feeds/'
      path: '/'
      fullPath: '/feeds/'
      preLoaderRoute: typeof PrivateFeedsIndexImport
      parentRoute: typeof PrivateFeedsImport
    }
  }
}

// Create and export the route tree

export const routeTree = rootRoute.addChildren({
  PrivateRoute: PrivateRoute.addChildren({
    PrivateBookmarksRoute: PrivateBookmarksRoute.addChildren({
      PrivateBookmarksStashRoute,
      PrivateBookmarksIndexRoute,
    }),
    PrivateFeedsRoute: PrivateFeedsRoute.addChildren({
      PrivateFeedsIdRoute,
      PrivateFeedsArchivedRoute,
      PrivateFeedsIndexRoute,
    }),
    PrivateIndexRoute,
  }),
  LoginRoute,
})

/* prettier-ignore-end */

/* ROUTE_MANIFEST_START
{
  "routes": {
    "__root__": {
      "filePath": "__root.tsx",
      "children": [
        "/_private",
        "/login"
      ]
    },
    "/_private": {
      "filePath": "_private.tsx",
      "children": [
        "/_private/bookmarks",
        "/_private/feeds",
        "/_private/"
      ]
    },
    "/login": {
      "filePath": "login.tsx"
    },
    "/_private/bookmarks": {
      "filePath": "_private/bookmarks.tsx",
      "parent": "/_private",
      "children": [
        "/_private/bookmarks/stash",
        "/_private/bookmarks/"
      ]
    },
    "/_private/feeds": {
      "filePath": "_private/feeds.tsx",
      "parent": "/_private",
      "children": [
        "/_private/feeds/$id",
        "/_private/feeds/archived",
        "/_private/feeds/"
      ]
    },
    "/_private/": {
      "filePath": "_private/index.tsx",
      "parent": "/_private"
    },
    "/_private/bookmarks/stash": {
      "filePath": "_private/bookmarks/stash.tsx",
      "parent": "/_private/bookmarks"
    },
    "/_private/feeds/$id": {
      "filePath": "_private/feeds/$id.tsx",
      "parent": "/_private/feeds"
    },
    "/_private/feeds/archived": {
      "filePath": "_private/feeds/archived.tsx",
      "parent": "/_private/feeds"
    },
    "/_private/bookmarks/": {
      "filePath": "_private/bookmarks/index.tsx",
      "parent": "/_private/bookmarks"
    },
    "/_private/feeds/": {
      "filePath": "_private/feeds/index.tsx",
      "parent": "/_private/feeds"
    }
  }
}
ROUTE_MANIFEST_END */
