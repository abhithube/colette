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
  tags: z.union([z.array(Tag), z.undefined()]).optional(),
});

export type BookmarkCreate = z.infer<typeof BookmarkCreate>;
export const BookmarkCreate = z.object({
  url: z.string(),
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
  tags: z.array(z.string()).optional(),
});

export type BooleanOperation = z.infer<typeof BooleanOperation>;
export const BooleanOperation = z.object({
  value: z.boolean(),
});

export type DateOperation = z.infer<typeof DateOperation>;
export const DateOperation = z.union([
  z.object({
    value: z.string(),
    type: z.literal("equals"),
  }),
  z.object({
    value: z.string(),
    type: z.literal("greaterThan"),
  }),
  z.object({
    value: z.string(),
    type: z.literal("lessThan"),
  }),
  z.object({
    value: z.number(),
    type: z.literal("inLast"),
  }),
]);

export type Feed = z.infer<typeof Feed>;
export const Feed = z.object({
  id: z.string(),
  link: z.string(),
  title: z.union([z.string(), z.null()]),
  pinned: z.boolean(),
  originalTitle: z.string(),
  url: z.union([z.string(), z.null()]),
  tags: z.union([z.array(Tag), z.undefined()]).optional(),
  unreadCount: z.union([z.number(), z.undefined()]).optional(),
});

export type FeedCreate = z.infer<typeof FeedCreate>;
export const FeedCreate = z.object({
  url: z.string(),
  pinned: z.union([z.boolean(), z.undefined()]).optional(),
  tags: z.union([z.array(z.string()), z.undefined()]).optional(),
});

export type FeedDetect = z.infer<typeof FeedDetect>;
export const FeedDetect = z.object({
  url: z.string(),
});

export type FeedDetected = z.infer<typeof FeedDetected>;
export const FeedDetected = z.object({
  url: z.string(),
  title: z.string(),
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
  pinned: z.boolean().optional(),
  tags: z.array(z.string()).optional(),
});

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
      title: z.union([z.string(), z.null()]),
      pinned: z.boolean(),
      originalTitle: z.string(),
      url: z.union([z.string(), z.null()]),
      tags: z.union([z.array(Tag), z.undefined()]).optional(),
      unreadCount: z.union([z.number(), z.undefined()]).optional(),
    }),
  ),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type Paginated_FeedDetected = z.infer<typeof Paginated_FeedDetected>;
export const Paginated_FeedDetected = z.object({
  data: z.array(
    z.object({
      url: z.string(),
      title: z.string(),
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

export type Paginated_SmartFeed = z.infer<typeof Paginated_SmartFeed>;
export const Paginated_SmartFeed = z.object({
  data: z.array(
    z.object({
      id: z.string(),
      title: z.string(),
      unreadCount: z.union([z.number(), z.undefined()]).optional(),
    }),
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

export type SmartFeed = z.infer<typeof SmartFeed>;
export const SmartFeed = z.object({
  id: z.string(),
  title: z.string(),
  unreadCount: z.union([z.number(), z.undefined()]).optional(),
});

export type TextOperation = z.infer<typeof TextOperation>;
export const TextOperation = z.union([
  z.object({
    value: z.string(),
    type: z.literal("equals"),
  }),
  z.object({
    value: z.string(),
    type: z.literal("doesNotEqual"),
  }),
  z.object({
    value: z.string(),
    type: z.literal("contains"),
  }),
  z.object({
    value: z.string(),
    type: z.literal("doesNotContain"),
  }),
]);

export type SmartFeedFilter = z.infer<typeof SmartFeedFilter>;
export const SmartFeedFilter = z.union([
  z.object({
    operation: TextOperation,
    field: z.literal("link"),
  }),
  z.object({
    operation: TextOperation,
    field: z.literal("title"),
  }),
  z.object({
    operation: DateOperation,
    field: z.literal("publishedAt"),
  }),
  z.object({
    operation: TextOperation,
    field: z.literal("description"),
  }),
  z.object({
    operation: TextOperation,
    field: z.literal("author"),
  }),
  z.object({
    operation: BooleanOperation,
    field: z.literal("hasRead"),
  }),
]);

export type SmartFeedCreate = z.infer<typeof SmartFeedCreate>;
export const SmartFeedCreate = z.object({
  title: z.string(),
  filters: z.union([z.array(SmartFeedFilter), z.undefined()]).optional(),
});

export type SmartFeedUpdate = z.infer<typeof SmartFeedUpdate>;
export const SmartFeedUpdate = z.object({
  title: z.union([z.string(), z.null()]).optional(),
  filters: z.array(SmartFeedFilter).optional(),
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
      pinned: z.boolean().optional(),
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
  response: Paginated_FeedDetected,
};

export type get_ListSmartFeeds = typeof get_ListSmartFeeds;
export const get_ListSmartFeeds = {
  method: z.literal("GET"),
  path: z.literal("/smartFeeds"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: Paginated_SmartFeed,
};

export type post_CreateSmartFeed = typeof post_CreateSmartFeed;
export const post_CreateSmartFeed = {
  method: z.literal("POST"),
  path: z.literal("/smartFeeds"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: SmartFeedCreate,
  }),
  response: SmartFeed,
};

export type get_GetSmartFeed = typeof get_GetSmartFeed;
export const get_GetSmartFeed = {
  method: z.literal("GET"),
  path: z.literal("/smartFeeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: SmartFeed,
};

export type delete_DeleteSmartFeed = typeof delete_DeleteSmartFeed;
export const delete_DeleteSmartFeed = {
  method: z.literal("DELETE"),
  path: z.literal("/smartFeeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type patch_UpdateSmartFeed = typeof patch_UpdateSmartFeed;
export const patch_UpdateSmartFeed = {
  method: z.literal("PATCH"),
  path: z.literal("/smartFeeds/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: SmartFeedUpdate,
  }),
  response: SmartFeed,
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
    "/smartFeeds": post_CreateSmartFeed,
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
    "/smartFeeds": get_ListSmartFeeds,
    "/smartFeeds/{id}": get_GetSmartFeed,
    "/tags": get_ListTags,
    "/tags/{id}": get_GetTag,
  },
  delete: {
    "/bookmarks/{id}": delete_DeleteBookmark,
    "/feeds/{id}": delete_DeleteFeed,
    "/smartFeeds/{id}": delete_DeleteSmartFeed,
    "/tags/{id}": delete_DeleteTag,
  },
  patch: {
    "/bookmarks/{id}": patch_UpdateBookmark,
    "/feedEntries/{id}": patch_UpdateFeedEntry,
    "/feeds/{id}": patch_UpdateFeed,
    "/smartFeeds/{id}": patch_UpdateSmartFeed,
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
