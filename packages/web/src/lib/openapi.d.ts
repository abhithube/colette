/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
    "/api/v1/auth/register": {
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
    "/api/v1/auth/login": {
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
    "/api/v1/bookmarks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the active profile bookmarks */
        get: operations["listBookmarks"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/bookmarks/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** @description Delete a bookmark by ID */
        delete: operations["deleteBookmark"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/collections": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List the active profile collections */
        get: operations["listCollections"];
        put?: never;
        /** @description Create a bookmarks collection */
        post: operations["createCollection"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/collections/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Get a collection by ID */
        get: operations["getCollection"];
        put?: never;
        post?: never;
        /** @description Delete a collection by ID */
        delete: operations["deleteCollection"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/entries": {
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
    "/api/v1/feeds": {
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
    "/api/v1/feeds/{id}": {
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
        patch?: never;
        trace?: never;
    };
    "/api/v1/profiles": {
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
    "/api/v1/profiles/@me": {
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
    "/api/v1/profiles/{id}": {
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
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        BaseError: {
            message: string;
        };
        Bookmark: {
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: uri */
            thumbnailUrl?: string | null;
            /** Format: date-time */
            publishedAt?: string | null;
            author?: string | null;
            customTitle?: string | null;
            /** Format: uri */
            customThumbnailUrl?: string | null;
            /** Format: date-time */
            customPublishedAt?: string | null;
            customAuthor?: string | null;
            collectionId: string;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            updatedAt: string;
        };
        BookmarkList: {
            hasMore: boolean;
            data: components["schemas"]["Bookmark"][];
        };
        Collection: {
            id: string;
            title: string;
            profileId: string;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            updatedAt: string;
        };
        CollectionList: {
            hasMore: boolean;
            data: components["schemas"]["Collection"][];
        };
        CreateCollection: {
            title: string;
        };
        CreateFeed: {
            url: string;
        };
        CreateProfile: {
            title: string;
            imageUrl?: string;
        };
        Entry: {
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: date-time */
            publishedAt?: string | null;
            description?: string | null;
            author?: string | null;
            /** Format: uri */
            thumbnailUrl?: string | null;
            hasRead: boolean;
            feedId: string;
        };
        EntryList: {
            hasMore: boolean;
            data: components["schemas"]["Entry"][];
        };
        Feed: {
            id: string;
            /** Format: uri */
            link: string;
            title: string;
            /** Format: uri */
            url?: string | null;
            customTitle?: string | null;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            updatedAt: string;
            /** Format: int64 */
            unreadCount?: number | null;
        };
        FeedList: {
            hasMore: boolean;
            data: components["schemas"]["Feed"][];
        };
        Login: {
            /** Format: email */
            email: string;
            password: string;
        };
        Profile: {
            id: string;
            title: string;
            /** Format: uri */
            imageUrl?: string | null;
            userId: string;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            updatedAt: string;
        };
        ProfileList: {
            hasMore: boolean;
            data: components["schemas"]["Profile"][];
        };
        Register: {
            /** Format: email */
            email: string;
            password: string;
        };
        User: {
            id: string;
            /** Format: email */
            email: string;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            updatedAt: string;
        };
        ValidationError: {
            code: string;
            message: string;
            params: {
                [key: string]: string | undefined;
            };
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
                    "application/json": {
                        email?: components["schemas"]["ValidationError"][] | null;
                        password?: components["schemas"]["ValidationError"][] | null;
                    };
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
                    "application/json": {
                        email?: components["schemas"]["ValidationError"][] | null;
                        password?: components["schemas"]["ValidationError"][] | null;
                    };
                };
            };
        };
    };
    listBookmarks: {
        parameters: {
            query?: {
                publishedAt?: string | null;
                collectionId?: string | null;
                isDefault?: boolean | null;
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
    listCollections: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Paginated list of collections */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CollectionList"];
                };
            };
        };
    };
    createCollection: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateCollection"];
            };
        };
        responses: {
            /** @description Created collection */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Collection"];
                };
            };
            /** @description Invalid input */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": {
                        title?: components["schemas"]["ValidationError"][] | null;
                    };
                };
            };
        };
    };
    getCollection: {
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
            /** @description Collection by ID */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Collection"];
                };
            };
            /** @description Collection not found */
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
    deleteCollection: {
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
            /** @description Successfully deleted collection */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Collection not found */
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
    listEntries: {
        parameters: {
            query?: {
                publishedAt?: string | null;
                feedId?: string | null;
                hasRead?: boolean | null;
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
                "application/json": components["schemas"]["CreateFeed"];
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
                    "application/json": {
                        url?: components["schemas"]["ValidationError"][] | null;
                    };
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
                "application/json": components["schemas"]["CreateProfile"];
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
                    "application/json": {
                        title?: components["schemas"]["ValidationError"][] | null;
                        imageUrl?: components["schemas"]["ValidationError"][] | null;
                    };
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
}
