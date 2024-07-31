/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
    "/auth/register": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Register a user account */
        post: operations["register"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/auth/login": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Login to a user account */
        post: operations["login"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/bookmarks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the active profile bookmarks */
        get: operations["listBookmarks"];
        put?: never;
        /** @description Add a bookmark to a profile */
        post: operations["createBookmark"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/bookmarks/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Get a bookmark by ID */
        get: operations["getBookmark"];
        put?: never;
        post?: never;
        /** @description Delete a bookmark by ID */
        delete: operations["deleteBookmark"];
        options?: never;
        head?: never;
        /** @description Update a bookmark by ID */
        patch: operations["updateBookmark"];
        trace?: never;
    };
    "/entries": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List feed entries */
        get: operations["listEntries"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/entries/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        /** @description Update a feed entry by ID */
        patch: operations["updateEntry"];
        trace?: never;
    };
    "/feeds": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the active profile feeds */
        get: operations["listFeeds"];
        put?: never;
        /** @description Subscribe to a web feed */
        post: operations["createFeed"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/feeds/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Get a feed by ID */
        get: operations["getFeed"];
        put?: never;
        post?: never;
        /** @description Delete a feed by ID */
        delete: operations["deleteFeed"];
        options?: never;
        head?: never;
        /** @description Update a feed by ID */
        patch: operations["updateFeed"];
        trace?: never;
    };
    "/feeds/detect": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Detects web feeds on a page */
        post: operations["detectFeeds"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/feeds/import": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Import OPML feeds into profile */
        post: operations["importFeeds"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/feeds/export": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Export OPML feeds from profile */
        post: operations["exportFeeds"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/profiles": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the user profiles */
        get: operations["listProfiles"];
        put?: never;
        /** @description Create a user profile */
        post: operations["createProfile"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/profiles/@me": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Get the active profile */
        get: operations["getActiveProfile"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/profiles/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** @description Delete a profile by ID */
        delete: operations["deleteProfile"];
        options?: never;
        head?: never;
        /** @description Update a profile by ID */
        patch: operations["updateProfile"];
        trace?: never;
    };
    "/tags": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the active profile tags */
        get: operations["listTags"];
        put?: never;
        /** @description Create a tag */
        post: operations["createTag"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/tags/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Get a tag by ID */
        get: operations["getTag"];
        put?: never;
        post?: never;
        /** @description Delete a tag by ID */
        delete: operations["deleteTag"];
        options?: never;
        head?: never;
        /** @description Update a tag by ID */
        patch: operations["updateTag"];
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        /** Error */
        BaseError: {
            message: string;
        };
        Bookmark: {
            /** Format: uuid */
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: uri */
            thumbnailUrl: string | null;
            /** Format: date-time */
            publishedAt: string | null;
            author: string | null;
            tags?: components["schemas"]["Tag"][];
        };
        BookmarkCreate: {
            /** Format: uri */
            url: string;
        };
        BookmarkList: {
            hasMore: boolean;
            data: components["schemas"]["Bookmark"][];
        };
        BookmarkUpdate: {
            tags?: string[] | null;
        };
        Entry: {
            /** Format: uuid */
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: date-time */
            publishedAt: string | null;
            description: string | null;
            author: string | null;
            /** Format: uri */
            thumbnailUrl: string | null;
            hasRead: boolean;
            /** Format: uuid */
            feedId: string;
        };
        EntryList: {
            hasMore: boolean;
            data: components["schemas"]["Entry"][];
        };
        EntryUpdate: {
            hasRead?: boolean | null;
        };
        Feed: {
            /** Format: uuid */
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: uri */
            url: string | null;
            tags?: components["schemas"]["Tag"][];
            /** Format: int64 */
            unreadCount?: number;
        };
        FeedCreate: {
            /** Format: uri */
            url: string;
        };
        FeedDetect: {
            /** Format: uri */
            url: string;
        };
        FeedDetected: {
            /** Format: uri */
            url: string;
            title: string;
        };
        FeedDetectedList: {
            hasMore: boolean;
            data: components["schemas"]["FeedDetected"][];
        };
        FeedList: {
            hasMore: boolean;
            data: components["schemas"]["Feed"][];
        };
        FeedUpdate: {
            tags?: string[] | null;
        };
        File: {
            /** Format: Binary */
            data: string;
        };
        Login: {
            /** Format: email */
            email: string;
            password: string;
        };
        Profile: {
            /** Format: uuid */
            id: string;
            title: string;
            /** Format: uri */
            imageUrl: string | null;
            /** Format: uuid */
            userId: string;
        };
        ProfileCreate: {
            title: string;
            imageUrl?: string;
        };
        ProfileList: {
            hasMore: boolean;
            data: components["schemas"]["Profile"][];
        };
        ProfileUpdate: {
            title?: string;
            imageUrl?: string;
        };
        Register: {
            /** Format: email */
            email: string;
            password: string;
        };
        Tag: {
            /** Format: uuid */
            id: string;
            title: string;
        };
        TagCreate: {
            title: string;
        };
        TagList: {
            hasMore: boolean;
            data: components["schemas"]["Tag"][];
        };
        TagUpdate: {
            title?: string;
        };
        User: {
            /** Format: uuid */
            id: string;
            /** Format: email */
            email: string;
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    register: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["Register"];
            };
        };
        responses: {
            /** @description Registered user */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["User"];
                };
            };
            /** @description Email already registered */
            409: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    login: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["Login"];
            };
        };
        responses: {
            /** @description Default profile */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description Bad credentials */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    listBookmarks: {
        parameters: {
            query?: {
                publishedAt?: string;
                withTags?: boolean;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of bookmarks */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BookmarkList"];
                };
            };
        };
    };
    createBookmark: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["BookmarkCreate"];
            };
        };
        responses: {
            /** @description Created bookmark */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Bookmark"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Failed to fetch or parse bookmark */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    getBookmark: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Bookmark by ID */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Bookmark"];
                };
            };
            /** @description Bookmark not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    deleteBookmark: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Successfully deleted bookmark */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Bookmark not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    updateBookmark: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["BookmarkUpdate"];
            };
        };
        responses: {
            /** @description Updated bookmark */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Bookmark"];
                };
            };
            /** @description Bookmark not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    listEntries: {
        parameters: {
            query?: {
                publishedAt?: string;
                feedId?: string;
                hasRead?: boolean;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of entries */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EntryList"];
                };
            };
        };
    };
    updateEntry: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["EntryUpdate"];
            };
        };
        responses: {
            /** @description Updated entry */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Entry"];
                };
            };
            /** @description Entry not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    listFeeds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of profiles */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FeedList"];
                };
            };
        };
    };
    createFeed: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["FeedCreate"];
            };
        };
        responses: {
            /** @description Created feed */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Feed"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Failed to fetch or parse feed */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    getFeed: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Feed by ID */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Feed"];
                };
            };
            /** @description Feed not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    deleteFeed: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Successfully deleted feed */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Feed not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    updateFeed: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["FeedUpdate"];
            };
        };
        responses: {
            /** @description Updated feed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Feed"];
                };
            };
            /** @description Feed not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    detectFeeds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["FeedDetect"];
            };
        };
        responses: {
            /** @description Detected feeds */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FeedDetectedList"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Failed to fetch or parse feed */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    importFeeds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "multipart/form-data": components["schemas"]["File"];
            };
        };
        responses: {
            /** @description Successfully started import */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    exportFeeds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OPML file */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
        };
    };
    listProfiles: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of profiles */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ProfileList"];
                };
            };
        };
    };
    createProfile: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ProfileCreate"];
            };
        };
        responses: {
            /** @description Created profile */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    getActiveProfile: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Active profile */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
        };
    };
    deleteProfile: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Successfully deleted profile */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Profile not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Deleting default profile */
            409: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    updateProfile: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ProfileUpdate"];
            };
        };
        responses: {
            /** @description Updated profile */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description Profile not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    listTags: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of tags */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TagList"];
                };
            };
        };
    };
    createTag: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["TagCreate"];
            };
        };
        responses: {
            /** @description Created tag */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Tag"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    getTag: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Tag by ID */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Tag"];
                };
            };
            /** @description Tag not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    deleteTag: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Successfully deleted tag */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Tag not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
    updateTag: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["TagUpdate"];
            };
        };
        responses: {
            /** @description Updated tag */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Tag"];
                };
            };
            /** @description Tag not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BaseError"];
                };
            };
        };
    };
}
