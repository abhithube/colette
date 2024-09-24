import { z } from "zod";

export type BaseError = z.infer<typeof BaseError>;
export const BaseError = z.object({
  message: z.string(),
});

export type Tag = z.infer<typeof Tag>;
export const Tag = z.object({
  id: z.string(),
  title: z.string(),
  parentId: z.union([z.string(), z.null()]),
  depth: z.number(),
  direct: z.union([z.boolean(), z.undefined()]).optional(),
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
  sortIndex: z.number(),
  tags: z.union([z.array(Tag), z.undefined()]).optional(),
});

export type TagsLink = z.infer<typeof TagsLink>;
export const TagsLink = z.object({
  data: z.array(z.string()),
  action: z.union([z.literal("add"), z.literal("set"), z.literal("remove")]),
});

export type BookmarkCreate = z.infer<typeof BookmarkCreate>;
export const BookmarkCreate = z.object({
  url: z.string(),
  tags: z.union([TagsLink, z.undefined()]).optional(),
});

export type BookmarkList = z.infer<typeof BookmarkList>;
export const BookmarkList = z.object({
  data: z.array(Bookmark),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type BookmarkUpdate = z.infer<typeof BookmarkUpdate>;
export const BookmarkUpdate = z.object({
  sortIndex: z.number().optional(),
  tags: TagsLink.optional(),
});

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
  tags: z.union([TagsLink, z.undefined()]).optional(),
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

export type FeedDetectedList = z.infer<typeof FeedDetectedList>;
export const FeedDetectedList = z.object({
  data: z.array(FeedDetected),
  cursor: z.union([z.string(), z.undefined()]).optional(),
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

export type FeedEntryList = z.infer<typeof FeedEntryList>;
export const FeedEntryList = z.object({
  data: z.array(FeedEntry),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type FeedEntryUpdate = z.infer<typeof FeedEntryUpdate>;
export const FeedEntryUpdate = z.object({
  hasRead: z.union([z.boolean(), z.null()]).optional(),
});

export type FeedList = z.infer<typeof FeedList>;
export const FeedList = z.object({
  data: z.array(Feed),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type FeedUpdate = z.infer<typeof FeedUpdate>;
export const FeedUpdate = z.object({
  title: z.union([z.string(), z.null()]).optional(),
  pinned: z.boolean().optional(),
  tags: TagsLink.optional(),
});

export type Login = z.infer<typeof Login>;
export const Login = z.object({
  email: z.string(),
  password: z.string(),
});

export type Profile = z.infer<typeof Profile>;
export const Profile = z.object({
  id: z.string(),
  title: z.string(),
  imageUrl: z.union([z.string(), z.null()]),
  isDefault: z.boolean(),
  userId: z.string(),
});

export type ProfileCreate = z.infer<typeof ProfileCreate>;
export const ProfileCreate = z.object({
  title: z.string(),
  imageUrl: z.union([z.string(), z.null(), z.undefined()]).optional(),
});

export type ProfileList = z.infer<typeof ProfileList>;
export const ProfileList = z.object({
  data: z.array(Profile),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type ProfileUpdate = z.infer<typeof ProfileUpdate>;
export const ProfileUpdate = z.object({
  title: z.string().optional(),
  imageUrl: z.union([z.string(), z.null()]).optional(),
});

export type Register = z.infer<typeof Register>;
export const Register = z.object({
  email: z.string(),
  password: z.string(),
});

export type SwitchProfile = z.infer<typeof SwitchProfile>;
export const SwitchProfile = z.object({
  id: z.string(),
});

export type TagCreate = z.infer<typeof TagCreate>;
export const TagCreate = z.object({
  title: z.string(),
  parentId: z.union([z.string(), z.null(), z.undefined()]).optional(),
});

export type TagList = z.infer<typeof TagList>;
export const TagList = z.object({
  data: z.array(Tag),
  cursor: z.union([z.string(), z.undefined()]).optional(),
});

export type TagUpdate = z.infer<typeof TagUpdate>;
export const TagUpdate = z.object({
  title: z.string().optional(),
  parentId: z.union([z.string(), z.null()]).optional(),
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
  response: Profile,
};

export type get_GetActiveUser = typeof get_GetActiveUser;
export const get_GetActiveUser = {
  method: z.literal("GET"),
  path: z.literal("/auth/@me"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: User,
};

export type post_SwitchProfile = typeof post_SwitchProfile;
export const post_SwitchProfile = {
  method: z.literal("POST"),
  path: z.literal("/auth/switchProfile"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: SwitchProfile,
  }),
  response: Profile,
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
  response: BookmarkList,
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

export type get_ListFeedEntries = typeof get_ListFeedEntries;
export const get_ListFeedEntries = {
  method: z.literal("GET"),
  path: z.literal("/feedEntries"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    query: z.object({
      feedId: z.string().optional(),
      hasRead: z.boolean().optional(),
      "tag[]": z.array(z.string()).optional(),
      cursor: z.string().optional(),
    }),
  }),
  response: FeedEntryList,
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
  response: FeedList,
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

export type post_DetectFeeds = typeof post_DetectFeeds;
export const post_DetectFeeds = {
  method: z.literal("POST"),
  path: z.literal("/feeds/detect"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: FeedDetect,
  }),
  response: FeedDetectedList,
};

export type get_ListProfiles = typeof get_ListProfiles;
export const get_ListProfiles = {
  method: z.literal("GET"),
  path: z.literal("/profiles"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: ProfileList,
};

export type post_CreateProfile = typeof post_CreateProfile;
export const post_CreateProfile = {
  method: z.literal("POST"),
  path: z.literal("/profiles"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: ProfileCreate,
  }),
  response: Profile,
};

export type get_GetProfile = typeof get_GetProfile;
export const get_GetProfile = {
  method: z.literal("GET"),
  path: z.literal("/profiles/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: Profile,
};

export type patch_UpdateProfile = typeof patch_UpdateProfile;
export const patch_UpdateProfile = {
  method: z.literal("PATCH"),
  path: z.literal("/profiles/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
    body: ProfileUpdate,
  }),
  response: Profile,
};

export type delete_DeleteProfile = typeof delete_DeleteProfile;
export const delete_DeleteProfile = {
  method: z.literal("DELETE"),
  path: z.literal("/profiles/{id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type get_GetActiveProfile = typeof get_GetActiveProfile;
export const get_GetActiveProfile = {
  method: z.literal("GET"),
  path: z.literal("/profiles/@me"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: Profile,
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
  response: TagList,
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

// <EndpointByMethod>
export const EndpointByMethod = {
  post: {
    "/auth/register": post_Register,
    "/auth/login": post_Login,
    "/auth/switchProfile": post_SwitchProfile,
    "/auth/logout": post_Logout,
    "/backups/opml/import": post_ImportOpml,
    "/backups/opml/export": post_ExportOpml,
    "/backups/netscape/import": post_ImportNetscape,
    "/backups/netscape/export": post_ExportNetscape,
    "/bookmarks": post_CreateBookmark,
    "/feeds": post_CreateFeed,
    "/feeds/detect": post_DetectFeeds,
    "/profiles": post_CreateProfile,
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
    "/profiles": get_ListProfiles,
    "/profiles/{id}": get_GetProfile,
    "/profiles/@me": get_GetActiveProfile,
    "/tags": get_ListTags,
    "/tags/{id}": get_GetTag,
  },
  patch: {
    "/bookmarks/{id}": patch_UpdateBookmark,
    "/feedEntries/{id}": patch_UpdateFeedEntry,
    "/feeds/{id}": patch_UpdateFeed,
    "/profiles/{id}": patch_UpdateProfile,
    "/tags/{id}": patch_UpdateTag,
  },
  delete: {
    "/bookmarks/{id}": delete_DeleteBookmark,
    "/feeds/{id}": delete_DeleteFeed,
    "/profiles/{id}": delete_DeleteProfile,
    "/tags/{id}": delete_DeleteTag,
  },
};
export type EndpointByMethod = typeof EndpointByMethod;
// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type PostEndpoints = EndpointByMethod["post"];
export type GetEndpoints = EndpointByMethod["get"];
export type PatchEndpoints = EndpointByMethod["patch"];
export type DeleteEndpoints = EndpointByMethod["delete"];
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
export type Method = "get" | "head" | MutationMethod;

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

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("patch", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.patch>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("delete", this.baseUrl + path, params[0]) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.delete>
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
