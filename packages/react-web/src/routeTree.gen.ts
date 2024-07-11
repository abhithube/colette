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
    '/_private/': {
      id: '/_private/'
      path: '/'
      fullPath: '/'
      preLoaderRoute: typeof PrivateIndexImport
      parentRoute: typeof PrivateImport
    }
  }
}

// Create and export the route tree

export const routeTree = rootRoute.addChildren({
  PrivateRoute: PrivateRoute.addChildren({ PrivateIndexRoute }),
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
        "/_private/"
      ]
    },
    "/login": {
      "filePath": "login.tsx"
    },
    "/_private/": {
      "filePath": "_private/index.tsx",
      "parent": "/_private"
    }
  }
}
ROUTE_MANIFEST_END */
