import { z } from "zod";

export type BaseError = z.infer<typeof BaseError>;
export const BaseError = z.object({
  message: z.string(),
});

export type Tag = z.infer<typeof Tag>;
export const Tag = z.object({
  id: z.string(),
  title: z.string(),
  bookmarkCount: z.union([z.number(), z.undefined()]).optional(),
  feedCount: z.union([z.number(), z.undefined()]).optional(),
});

export type Bookmark = z.infer<typeof Bookmark>;
export const Bookmark = z.object({
  id: z.string(),
  link: z.string(),
  title: z.string(),
  thumbnailUrl: z.union([z.string(), z.null()]),
  publishedAt: z.union([z.string(), z.null()]),
  author: z.union([z.string(), z.null()]),
  archivedUrl: z.union([z.string(), z.null()]),
  folderId: z.union([z.string(), z.null()]),
  tags: z.union([z.array(Tag), z.undefined()]).optional(),
});

export type BookmarkCreate = z.infer<typeof BookmarkCreate>;
export const BookmarkCreate = z.object({
  url: z.string(),
  title: z.string(),
  thumbnailUrl: z.union([z.string(), z.null(), z.undefined()]).optional(),
  publishedAt: z.union([z.string(), z.null(), z.undefined()]).optional(),
  author: z.union([z.string(), z.null(), z.undefined()]).optional(),
  folderId: z.union([z.string(), z.null(), z.undefined()]).optional(),
  tags: z.union([z.array(z.string()), z.undefined()]).optional(),
});

export type BookmarkScrape = z.infer<typeof BookmarkScrape>;
export const BookmarkScrape = z.object({
  url: z.string(),
});

export type BookmarkScraped = z.infer<typeof BookmarkScraped>;
export const BookmarkScraped = z.object({
  link: z.string(),
  title: z.string(),
  thumbnailUrl: z.union([z.string(), z.null()]),
  publishedAt: z.union([z.string(), z.null()]),
  author: z.union([z.string(), z.null()]),
});

export type BookmarkUpdate = z.infer<typeof BookmarkUpdate>;
export const BookmarkUpdate = z.object({
  title: z.string().optional(),
  thumbnailUrl: z.union([z.string(), z.null()]).optional(),
  publishedAt: z.union([z.string(), z.null()]).optional(),
  author: z.union([z.string(), z.null()]).optional(),
  folderId: z.union([z.string(), z.null()]).optional(),
  tags: z.array(z.string()).optional(),
});

export type FeedDetected = z.infer<typeof FeedDetected>;
export const FeedDetected = z.object({
  url: z.string(),
  title: z.string(),
});

export type FeedProcessed = z.infer<typeof FeedProcessed>;
export const FeedProcessed = z.object({
  link: z.string(),
  title: z.string(),
});

export type DetectedResponse = z.infer<typeof DetectedResponse>;
export const DetectedResponse = z.union([z.array(FeedDetected), FeedProcessed]);

export type Feed = z.infer<typeof Feed>;
export const Feed = z.object({
  id: z.string(),
  link: z.string(),
  title: z.string(),
  xmlUrl: z.union([z.string(), z.null()]),
  folderId: z.union([z.string(), z.null()]),
  tags: z.union([z.array(Tag), z.undefined()]).optional(),
  unreadCount: z.union([z.number(), z.undefined()]).optional(),
});

export type FeedCreate = z.infer<typeof FeedCreate>;
export const FeedCreate = z.object({
  url: z.string(),
  title: z.string(),
  folderId: z.union([z.string(), z.null(), z.undefined()]).optional(),
  tags: z.union([z.array(z.string()), z.undefined()]).optional(),
});

export type FeedDetect = z.infer<typeof FeedDetect>;
export const FeedDetect = z.object({
  url: z.string(),
});

export type FeedEntry = z.infer<typeof FeedEntry>;
export const FeedEntry = z.object({
  id: z.string(),
  link: z.string(),
  title: z.string(),
  publishedAt: z.string(),
  description: z.union([z.string(), z.null()]),
  author: z.union([z.string(), z.null()]),
  thumbnailUrl: z.union([z.string(), z.null()]),
  hasRead: z.boolean(),
  feedId: z.string(),
});

export type FeedEntryUpdate = z.infer<typeof FeedEntryUpdate>;
export const FeedEntryUpdate = z.object({
  hasRead: z.union([z.boolean(), z.null()]).optional(),
});

export type FeedUpdate = z.infer<typeof FeedUpdate>;
export const FeedUpdate = z.object({
  title: z.union([z.string(), z.null()]).optional(),
  folderId: z.union([z.string(), z.null()]).optional(),
  tags: z.array(z.string()).optional(),
});

export type Folder = z.infer<typeof Folder>;
export const Folder = z.object({
  id: z.string(),
  title: z.string(),
  parentId: z.union([z.string(), z.undefined()]).optional(),
});

export type FolderCreate = z.infer<typeof FolderCreate>;
export const FolderCreate = z.object({
  title: z.string(),
  parentId: z.union([z.string(), z.null(), z.undefined()]).optional(),
});

export type FolderUpdate = z.infer<typeof FolderUpdate>;
export const FolderUpdate = z.object({
  title: z.string().optional(),
  parentId: z.union([z.string(), z.null()]).optional(),
});

export type LibraryItem = z.infer<typeof LibraryItem>;
export const LibraryItem = z.union([
  z.object({
    data: Folder,
    type: z.literal("folder"),
  }),
  z.object({
    data: Feed,
    type: z.literal("feed"),
  }),
  z.object({
    data: Bookmark,
    type: z.literal("bookmark"),
  }),
]);

export type Login = z.infer<typeof Login>;
export const Login = z.object({
  email: z.string(),
  password: z.string(),
});

export type Paginated_Bookmark = z.infer<typeof Paginated_Bookmark>;
export const Paginated_Bookmark = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      link: z.string(),
      title: z.string(),
      thumbnailUrl: z.union([z.string(), z.null()]),
      publishedAt: z.union([z.string(), z.null()]),
      author: z.union([z.string(), z.null()]),
      archivedUrl: z.union([z.string(), z.null()]),
      folderId: z.union([z.string(), z.null()]),
      tags: z.union([z.array(Tag), z.undefined()]).optional(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_Feed = z.infer<typeof Paginated_Feed>;
export const Paginated_Feed = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      link: z.string(),
      title: z.string(),
      xmlUrl: z.union([z.string(), z.null()]),
      folderId: z.union([z.string(), z.null()]),
      tags: z.union([z.array(Tag), z.undefined()]).optional(),
      unreadCount: z.union([z.number(), z.undefined()]).optional(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_FeedEntry = z.infer<typeof Paginated_FeedEntry>;
export const Paginated_FeedEntry = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      link: z.string(),
      title: z.string(),
      publishedAt: z.string(),
      description: z.union([z.string(), z.null()]),
      author: z.union([z.string(), z.null()]),
      thumbnailUrl: z.union([z.string(), z.null()]),
      hasRead: z.boolean(),
      feedId: z.string(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_Folder = z.infer<typeof Paginated_Folder>;
export const Paginated_Folder = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      title: z.string(),
      parentId: z.union([z.string(), z.undefined()]).optional(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_LibraryItem = z.infer<typeof Paginated_LibraryItem>;
export const Paginated_LibraryItem = z.object({
  data: z.array(
    z.union([
      z.object({
        data: Folder,
        type: z.literal("folder"),
      }),
      z.object({
        data: Feed,
        type: z.literal("feed"),
      }),
      z.object({
        data: Bookmark,
        type: z.literal("bookmark"),
      }),
    ]),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_Tag = z.infer<typeof Paginated_Tag>;
export const Paginated_Tag = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      title: z.string(),
      bookmarkCount: z.union([z.number(), z.undefined()]).optional(),
      feedCount: z.union([z.number(), z.undefined()]).optional(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Register = z.infer<typeof Register>;
export const Register = z.object({
  email: z.string(),
  password: z.string(),
});

export type TagCreate = z.infer<typeof TagCreate>;
export const TagCreate = z.object({
  title: z.string(),
});

export type TagUpdate = z.infer<typeof TagUpdate>;
export const TagUpdate = z.object({
  title: z.string().optional(),
});

export type User = z.infer<typeof User>;
export const User = z.object({
  id: z.string(),
  email: z.string(),
});

export type post_Register = typeof post_Register;
export const post_Register = {
  method: z.literal("POST"),
  path: z.literal("/auth/register"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: Register,
  }),
  response: User,
};

export type post_Login = typeof post_Login;
export const post_Login = {
  method: z.literal("POST"),
  path: z.literal("/auth/login"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: Login,
  }),
  response: User,
};

export type get_GetActiveUser = typeof get_GetActiveUser;
export const get_GetActiveUser = {
  method: z.literal("GET"),
  path: z.literal("/auth/@me"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: User,
};

export type post_Logout = typeof post_Logout;
export const post_Logout = {
  method: z.literal("POST"),
  path: z.literal("/auth/logout"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: z.unknown(),
};

export type post_ImportOpml = typeof post_ImportOpml;
export const post_ImportOpml = {
  method: z.literal("POST"),
  path: z.literal("/backups/opml/import"),
  requestFormat: z.literal("binary"),
  parameters: z.object({
    body: z.array(z.number()),
  }),
  response: z.unknown(),
};

export type post_ExportOpml = typeof post_ExportOpml;
export const post_ExportOpml = {
  method: z.literal("POST"),
  path: z.literal("/backups/opml/export"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: z.unknown(),
};

export type post_ImportNetscape = typeof post_ImportNetscape;
export const post_ImportNetscape = {
  method: z.literal("POST"),
  path: z.literal("/backups/netscape/import"),
  requestFormat: z.literal("binary"),
  parameters: z.object({
    body: z.array(z.number()),
  }),
  response: z.unknown(),
};

export type post_ExportNetscape = typeof post_ExportNetscape;
export const post_ExportNetscape = {
  method: z.literal("POST"),
  path: z.literal("/backups/netscape/export"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: z.unknown(),
};

export type get_ListBookmarks = typeof get_ListBookmarks;
export const get_ListBookmarks = {
  method: z.literal("GET"),
  path: z.literal("/bookmarks"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      filterByFolder: z.boolean().optional(),
      folderId: z.string().optional(),
      filterByTags: z.boolean().optional(),
      "tag[]": z.array(z.string()).optional(),
      cursor: z.string().optional(),
    }),
  }),
  response: Paginated_Bookmark,
};

export type post_CreateBookmark = typeof post_CreateBookmark;
export const post_CreateBookmark = {
  method: z.literal("POST"),
  path: z.literal("/bookmarks"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: BookmarkCreate,
  }),
  response: Bookmark,
};

export type get_GetBookmark = typeof get_GetBookmark;
export const get_GetBookmark = {
  method: z.literal("GET"),
  path: z.literal("/bookmarks/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: Bookmark,
};

export type delete_DeleteBookmark = typeof delete_DeleteBookmark;
export const delete_DeleteBookmark = {
  method: z.literal("DELETE"),
  path: z.literal("/bookmarks/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type patch_UpdateBookmark = typeof patch_UpdateBookmark;
export const patch_UpdateBookmark = {
  method: z.literal("PATCH"),
  path: z.literal("/bookmarks/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: BookmarkUpdate,
  }),
  response: Bookmark,
};

export type post_ScrapeBookmark = typeof post_ScrapeBookmark;
export const post_ScrapeBookmark = {
  method: z.literal("POST"),
  path: z.literal("/bookmarks/scrape"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: BookmarkScrape,
  }),
  response: BookmarkScraped,
};

export type get_ListFeedEntries = typeof get_ListFeedEntries;
export const get_ListFeedEntries = {
  method: z.literal("GET"),
  path: z.literal("/feedEntries"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      feedId: z.string().optional(),
      smartFeedId: z.string().optional(),
      hasRead: z.boolean().optional(),
      "tag[]": z.array(z.string()).optional(),
      cursor: z.string().optional(),
    }),
  }),
  response: Paginated_FeedEntry,
};

export type get_GetFeedEntry = typeof get_GetFeedEntry;
export const get_GetFeedEntry = {
  method: z.literal("GET"),
  path: z.literal("/feedEntries/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: FeedEntry,
};

export type patch_UpdateFeedEntry = typeof patch_UpdateFeedEntry;
export const patch_UpdateFeedEntry = {
  method: z.literal("PATCH"),
  path: z.literal("/feedEntries/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: FeedEntryUpdate,
  }),
  response: FeedEntry,
};

export type get_ListFeeds = typeof get_ListFeeds;
export const get_ListFeeds = {
  method: z.literal("GET"),
  path: z.literal("/feeds"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      filterByFolder: z.boolean().optional(),
      folderId: z.string().optional(),
      filterByTags: z.boolean().optional(),
      "tag[]": z.array(z.string()).optional(),
    }),
  }),
  response: Paginated_Feed,
};

export type post_CreateFeed = typeof post_CreateFeed;
export const post_CreateFeed = {
  method: z.literal("POST"),
  path: z.literal("/feeds"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: FeedCreate,
  }),
  response: Feed,
};

export type get_GetFeed = typeof get_GetFeed;
export const get_GetFeed = {
  method: z.literal("GET"),
  path: z.literal("/feeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: Feed,
};

export type delete_DeleteFeed = typeof delete_DeleteFeed;
export const delete_DeleteFeed = {
  method: z.literal("DELETE"),
  path: z.literal("/feeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type patch_UpdateFeed = typeof patch_UpdateFeed;
export const patch_UpdateFeed = {
  method: z.literal("PATCH"),
  path: z.literal("/feeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: FeedUpdate,
  }),
  response: Feed,
};

export type post_DetectFeeds = typeof post_DetectFeeds;
export const post_DetectFeeds = {
  method: z.literal("POST"),
  path: z.literal("/feeds/detect"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: FeedDetect,
  }),
  response: DetectedResponse,
};

export type get_ListFolders = typeof get_ListFolders;
export const get_ListFolders = {
  method: z.literal("GET"),
  path: z.literal("/folders"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      filterByParent: z.boolean().optional(),
      parentId: z.string().optional(),
    }),
  }),
  response: Paginated_Folder,
};

export type post_CreateFolder = typeof post_CreateFolder;
export const post_CreateFolder = {
  method: z.literal("POST"),
  path: z.literal("/folders"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: FolderCreate,
  }),
  response: Folder,
};

export type get_GetFolder = typeof get_GetFolder;
export const get_GetFolder = {
  method: z.literal("GET"),
  path: z.literal("/folders/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: Folder,
};

export type delete_DeleteFolder = typeof delete_DeleteFolder;
export const delete_DeleteFolder = {
  method: z.literal("DELETE"),
  path: z.literal("/folders/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type patch_UpdateFolder = typeof patch_UpdateFolder;
export const patch_UpdateFolder = {
  method: z.literal("PATCH"),
  path: z.literal("/folders/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: FolderUpdate,
  }),
  response: Folder,
};

export type get_ListLibraryItems = typeof get_ListLibraryItems;
export const get_ListLibraryItems = {
  method: z.literal("GET"),
  path: z.literal("/library"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      folderId: z.string().optional(),
      cursor: z.string().optional(),
    }),
  }),
  response: Paginated_LibraryItem,
};

export type get_ListTags = typeof get_ListTags;
export const get_ListTags = {
  method: z.literal("GET"),
  path: z.literal("/tags"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      tagType: z.union([z.literal("all"), z.literal("bookmarks"), z.literal("feeds")]).optional(),
    }),
  }),
  response: Paginated_Tag,
};

export type post_CreateTag = typeof post_CreateTag;
export const post_CreateTag = {
  method: z.literal("POST"),
  path: z.literal("/tags"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: TagCreate,
  }),
  response: Tag,
};

export type get_GetTag = typeof get_GetTag;
export const get_GetTag = {
  method: z.literal("GET"),
  path: z.literal("/tags/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: Tag,
};

export type delete_DeleteTag = typeof delete_DeleteTag;
export const delete_DeleteTag = {
  method: z.literal("DELETE"),
  path: z.literal("/tags/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type patch_UpdateTag = typeof patch_UpdateTag;
export const patch_UpdateTag = {
  method: z.literal("PATCH"),
  path: z.literal("/tags/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: TagUpdate,
  }),
  response: Tag,
};

// <EndpointByMethod>
export const EndpointByMethod = {
  post: {
    "/auth/register": post_Register,
    "/auth/login": post_Login,
    "/auth/logout": post_Logout,
    "/backups/opml/import": post_ImportOpml,
    "/backups/opml/export": post_ExportOpml,
    "/backups/netscape/import": post_ImportNetscape,
    "/backups/netscape/export": post_ExportNetscape,
    "/bookmarks": post_CreateBookmark,
    "/bookmarks/scrape": post_ScrapeBookmark,
    "/feeds": post_CreateFeed,
    "/feeds/detect": post_DetectFeeds,
    "/folders": post_CreateFolder,
    "/tags": post_CreateTag,
  },
  get: {
    "/auth/@me": get_GetActiveUser,
    "/bookmarks": get_ListBookmarks,
    "/bookmarks/{id}": get_GetBookmark,
    "/feedEntries": get_ListFeedEntries,
    "/feedEntries/{id}": get_GetFeedEntry,
    "/feeds": get_ListFeeds,
    "/feeds/{id}": get_GetFeed,
    "/folders": get_ListFolders,
    "/folders/{id}": get_GetFolder,
    "/library": get_ListLibraryItems,
    "/tags": get_ListTags,
    "/tags/{id}": get_GetTag,
  },
  delete: {
    "/bookmarks/{id}": delete_DeleteBookmark,
    "/feeds/{id}": delete_DeleteFeed,
    "/folders/{id}": delete_DeleteFolder,
    "/tags/{id}": delete_DeleteTag,
  },
  patch: {
    "/bookmarks/{id}": patch_UpdateBookmark,
    "/feedEntries/{id}": patch_UpdateFeedEntry,
    "/feeds/{id}": patch_UpdateFeed,
    "/folders/{id}": patch_UpdateFolder,
    "/tags/{id}": patch_UpdateTag,
  },
};
export type EndpointByMethod = typeof EndpointByMethod;
// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type PostEndpoints = EndpointByMethod["post"];
export type GetEndpoints = EndpointByMethod["get"];
export type DeleteEndpoints = EndpointByMethod["delete"];
export type PatchEndpoints = EndpointByMethod["patch"];
export type AllEndpoints = EndpointByMethod[keyof EndpointByMethod];
// </EndpointByMethod.Shorthands>

// <ApiClientTypes>
export type EndpointParameters = {
  body?: unknown;
  query?: Record<string, unknown>;
  header?: Record<string, unknown>;
  path?: Record<string, unknown>;
};

export type MutationMethod = "post" | "put" | "patch" | "delete";
export type Method = "get" | "head" | "options" | MutationMethod;

type RequestFormat = "json" | "form-data" | "form-url" | "binary" | "text";

export type DefaultEndpoint = {
  parameters?: EndpointParameters | undefined;
  response: unknown;
};

export type Endpoint<TConfig extends DefaultEndpoint = DefaultEndpoint> = {
  operationId: string;
  method: Method;
  path: string;
  requestFormat: RequestFormat;
  parameters?: TConfig["parameters"];
  meta: {
    alias: string;
    hasParameters: boolean;
    areParametersRequired: boolean;
  };
  response: TConfig["response"];
};

type Fetcher = (
  method: Method,
  url: string,
  parameters?: EndpointParameters | undefined,
) => Promise<Endpoint["response"]>;

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P;
}[keyof T];

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];

// </ApiClientTypes>

// <ApiClient>
export class ApiClient {
  baseUrl: string = "";

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl;
    return this;
  }

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("post", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.post>

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("get", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.get>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("delete", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.delete>

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("patch", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.patch>
}

export function createApiClient(fetcher: Fetcher, baseUrl?: string) {
  return new ApiClient(fetcher).setBaseUrl(baseUrl ?? "");
}

/**
 Example usage:
 const api = createApiClient((method, url, params) =>
   fetch(url, { method, body: JSON.stringify(params) }).then((res) => res.json()),
 );
 api.get("/users").then((users) => console.log(users));
 api.post("/users", { body: { name: "John" } }).then((user) => console.log(user));
 api.put("/users/:id", { path: { id: 1 }, body: { name: "John" } }).then((user) => console.log(user));
*/

// </ApiClient
