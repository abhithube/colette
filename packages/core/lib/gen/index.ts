export type {
  ApiError,
  ApiErrorCodeEnum,
  ApiErrorCode,
  ApiKey,
  ApiKeyCreate,
  ApiKeyCreated,
  ApiKeyUpdate,
  Bookmark,
  BookmarkCreate,
  BookmarkDateFieldEnum,
  BookmarkDateField,
  BookmarkDetails,
  BookmarkFilter,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkTextFieldEnum,
  BookmarkTextField,
  BookmarkUpdate,
  BooleanOp,
  Collection,
  CollectionCreate,
  CollectionUpdate,
  Config,
  DateOp,
  Feed,
  FeedDetect,
  FeedDetected,
  FeedEntry,
  FeedScrape,
  LinkBookmarkTags,
  LinkSubscriptionTags,
  OidcConfig,
  PaginatedApiKey,
  PaginatedBookmarkDetails,
  PaginatedCollection,
  PaginatedFeedEntry,
  PaginatedStream,
  PaginatedSubscriptionDetails,
  PaginatedSubscriptionEntryDetails,
  PaginatedTagDetails,
  StorageConfig,
  Stream,
  StreamCreate,
  StreamUpdate,
  Subscription,
  SubscriptionCreate,
  SubscriptionDetails,
  SubscriptionEntry,
  SubscriptionEntryBooleanFieldEnum,
  SubscriptionEntryBooleanField,
  SubscriptionEntryDateFieldEnum,
  SubscriptionEntryDateField,
  SubscriptionEntryDetails,
  SubscriptionEntryFilter,
  SubscriptionEntryTextFieldEnum,
  SubscriptionEntryTextField,
  SubscriptionUpdate,
  Tag,
  TagCreate,
  TagDetails,
  TagUpdate,
  TextOp,
  User,
  ListApiKeys200,
  ListApiKeys401,
  ListApiKeysError,
  ListApiKeysQueryResponse,
  ListApiKeysQuery,
  CreateApiKey201,
  CreateApiKey401,
  CreateApiKey422,
  CreateApiKeyError,
  CreateApiKeyMutationRequest,
  CreateApiKeyMutationResponse,
  CreateApiKeyMutation,
  GetApiKeyPathParams,
  GetApiKey200,
  GetApiKey401,
  GetApiKey403,
  GetApiKey404,
  GetApiKeyError,
  GetApiKeyQueryResponse,
  GetApiKeyQuery,
  DeleteApiKeyPathParams,
  DeleteApiKey204,
  DeleteApiKey401,
  DeleteApiKey403,
  DeleteApiKey404,
  DeleteApiKeyError,
  DeleteApiKeyMutationResponse,
  DeleteApiKeyMutation,
  UpdateApiKeyPathParams,
  UpdateApiKey200,
  UpdateApiKey401,
  UpdateApiKey403,
  UpdateApiKey404,
  UpdateApiKey422,
  UpdateApiKeyError,
  UpdateApiKeyMutationRequest,
  UpdateApiKeyMutationResponse,
  UpdateApiKeyMutation,
  GetActiveUser200,
  GetActiveUser401,
  GetActiveUserError,
  GetActiveUserQueryResponse,
  GetActiveUserQuery,
  ListBookmarksQueryParams,
  ListBookmarks200,
  ListBookmarks401,
  ListBookmarksError,
  ListBookmarksQueryResponse,
  ListBookmarksQuery,
  CreateBookmark201,
  CreateBookmark401,
  CreateBookmark409,
  CreateBookmark422,
  CreateBookmarkError,
  CreateBookmarkMutationRequest,
  CreateBookmarkMutationResponse,
  CreateBookmarkMutation,
  GetBookmarkPathParams,
  GetBookmarkQueryParams,
  GetBookmark200,
  GetBookmark401,
  GetBookmark403,
  GetBookmark404,
  GetBookmarkError,
  GetBookmarkQueryResponse,
  GetBookmarkQuery,
  DeleteBookmarkPathParams,
  DeleteBookmark204,
  DeleteBookmark401,
  DeleteBookmark403,
  DeleteBookmark404,
  DeleteBookmarkError,
  DeleteBookmarkMutationResponse,
  DeleteBookmarkMutation,
  UpdateBookmarkPathParams,
  UpdateBookmark200,
  UpdateBookmark401,
  UpdateBookmark403,
  UpdateBookmark404,
  UpdateBookmark422,
  UpdateBookmarkError,
  UpdateBookmarkMutationRequest,
  UpdateBookmarkMutationResponse,
  UpdateBookmarkMutation,
  LinkBookmarkTagsPathParams,
  LinkBookmarkTags204,
  LinkBookmarkTags401,
  LinkBookmarkTags403,
  LinkBookmarkTags404,
  LinkBookmarkTags422,
  LinkBookmarkTagsError,
  LinkBookmarkTagsMutationRequest,
  LinkBookmarkTagsMutationResponse,
  LinkBookmarkTagsMutation,
  ScrapeBookmark201,
  ScrapeBookmark401,
  ScrapeBookmark422,
  ScrapeBookmark502,
  ScrapeBookmarkError,
  ScrapeBookmarkMutationRequest,
  ScrapeBookmarkMutationResponse,
  ScrapeBookmarkMutation,
  ImportBookmarks204,
  ImportBookmarks401,
  ImportBookmarksError,
  ImportBookmarksMutationRequest,
  ImportBookmarksMutationResponse,
  ImportBookmarksMutation,
  ExportBookmarks200,
  ExportBookmarks401,
  ExportBookmarksError,
  ExportBookmarksMutationResponse,
  ExportBookmarksMutation,
  ListCollections200,
  ListCollections401,
  ListCollectionsError,
  ListCollectionsQueryResponse,
  ListCollectionsQuery,
  CreateCollection201,
  CreateCollection401,
  CreateCollection409,
  CreateCollection422,
  CreateCollectionError,
  CreateCollectionMutationRequest,
  CreateCollectionMutationResponse,
  CreateCollectionMutation,
  GetCollectionPathParams,
  GetCollection200,
  GetCollection401,
  GetCollection403,
  GetCollection404,
  GetCollectionError,
  GetCollectionQueryResponse,
  GetCollectionQuery,
  DeleteCollectionPathParams,
  DeleteCollection204,
  DeleteCollection401,
  DeleteCollection403,
  DeleteCollection404,
  DeleteCollectionError,
  DeleteCollectionMutationResponse,
  DeleteCollectionMutation,
  UpdateCollectionPathParams,
  UpdateCollection200,
  UpdateCollection401,
  UpdateCollection403,
  UpdateCollection404,
  UpdateCollection422,
  UpdateCollectionError,
  UpdateCollectionMutationRequest,
  UpdateCollectionMutationResponse,
  UpdateCollectionMutation,
  GetConfig200,
  GetConfigQueryResponse,
  GetConfigQuery,
  ListFeedEntriesQueryParams,
  ListFeedEntries200,
  ListFeedEntries401,
  ListFeedEntriesError,
  ListFeedEntriesQueryResponse,
  ListFeedEntriesQuery,
  GetFeedEntryPathParams,
  GetFeedEntry200,
  GetFeedEntry401,
  GetFeedEntry403,
  GetFeedEntry404,
  GetFeedEntryError,
  GetFeedEntryQueryResponse,
  GetFeedEntryQuery,
  DetectFeeds201,
  DetectFeeds401,
  DetectFeeds422,
  DetectFeeds502,
  DetectFeedsError,
  DetectFeedsMutationRequest,
  DetectFeedsMutationResponse,
  DetectFeedsMutation,
  ScrapeFeed201,
  ScrapeFeed401,
  ScrapeFeed422,
  ScrapeFeed502,
  ScrapeFeedError,
  ScrapeFeedMutationRequest,
  ScrapeFeedMutationResponse,
  ScrapeFeedMutation,
  ListStreams200,
  ListStreams401,
  ListStreamsError,
  ListStreamsQueryResponse,
  ListStreamsQuery,
  CreateStream201,
  CreateStream401,
  CreateStream409,
  CreateStream422,
  CreateStreamError,
  CreateStreamMutationRequest,
  CreateStreamMutationResponse,
  CreateStreamMutation,
  GetStreamPathParams,
  GetStream200,
  GetStream401,
  GetStream403,
  GetStream404,
  GetStreamError,
  GetStreamQueryResponse,
  GetStreamQuery,
  DeleteStreamPathParams,
  DeleteStream204,
  DeleteStream401,
  DeleteStream403,
  DeleteStream404,
  DeleteStreamError,
  DeleteStreamMutationResponse,
  DeleteStreamMutation,
  UpdateStreamPathParams,
  UpdateStream200,
  UpdateStream401,
  UpdateStream403,
  UpdateStream404,
  UpdateStream422,
  UpdateStreamError,
  UpdateStreamMutationRequest,
  UpdateStreamMutationResponse,
  UpdateStreamMutation,
  ListSubscriptionsQueryParams,
  ListSubscriptions200,
  ListSubscriptions401,
  ListSubscriptionsError,
  ListSubscriptionsQueryResponse,
  ListSubscriptionsQuery,
  CreateSubscription201,
  CreateSubscription401,
  CreateSubscription409,
  CreateSubscription422,
  CreateSubscriptionError,
  CreateSubscriptionMutationRequest,
  CreateSubscriptionMutationResponse,
  CreateSubscriptionMutation,
  GetSubscriptionPathParams,
  GetSubscriptionQueryParams,
  GetSubscription200,
  GetSubscription401,
  GetSubscription403,
  GetSubscription404,
  GetSubscriptionError,
  GetSubscriptionQueryResponse,
  GetSubscriptionQuery,
  DeleteSubscriptionPathParams,
  DeleteSubscription204,
  DeleteSubscription401,
  DeleteSubscription403,
  DeleteSubscription404,
  DeleteSubscriptionError,
  DeleteSubscriptionMutationResponse,
  DeleteSubscriptionMutation,
  UpdateSubscriptionPathParams,
  UpdateSubscription200,
  UpdateSubscription401,
  UpdateSubscription403,
  UpdateSubscription404,
  UpdateSubscription422,
  UpdateSubscriptionError,
  UpdateSubscriptionMutationRequest,
  UpdateSubscriptionMutationResponse,
  UpdateSubscriptionMutation,
  LinkSubscriptionTagsPathParams,
  LinkSubscriptionTags204,
  LinkSubscriptionTags401,
  LinkSubscriptionTags403,
  LinkSubscriptionTags404,
  LinkSubscriptionTags422,
  LinkSubscriptionTagsError,
  LinkSubscriptionTagsMutationRequest,
  LinkSubscriptionTagsMutationResponse,
  LinkSubscriptionTagsMutation,
  MarkSubscriptionEntryAsReadPathParams,
  MarkSubscriptionEntryAsRead200,
  MarkSubscriptionEntryAsRead401,
  MarkSubscriptionEntryAsRead403,
  MarkSubscriptionEntryAsRead404,
  MarkSubscriptionEntryAsRead422,
  MarkSubscriptionEntryAsReadError,
  MarkSubscriptionEntryAsReadMutationResponse,
  MarkSubscriptionEntryAsReadMutation,
  MarkSubscriptionEntryAsUnreadPathParams,
  MarkSubscriptionEntryAsUnread200,
  MarkSubscriptionEntryAsUnread401,
  MarkSubscriptionEntryAsUnread403,
  MarkSubscriptionEntryAsUnread404,
  MarkSubscriptionEntryAsUnread422,
  MarkSubscriptionEntryAsUnreadError,
  MarkSubscriptionEntryAsUnreadMutationResponse,
  MarkSubscriptionEntryAsUnreadMutation,
  ImportSubscriptions204,
  ImportSubscriptions401,
  ImportSubscriptionsError,
  ImportSubscriptionsMutationRequest,
  ImportSubscriptionsMutationResponse,
  ImportSubscriptionsMutation,
  ExportSubscriptions200,
  ExportSubscriptions401,
  ExportSubscriptionsError,
  ExportSubscriptionsMutationResponse,
  ExportSubscriptionsMutation,
  ListSubscriptionEntriesQueryParams,
  ListSubscriptionEntries200,
  ListSubscriptionEntries401,
  ListSubscriptionEntriesError,
  ListSubscriptionEntriesQueryResponse,
  ListSubscriptionEntriesQuery,
  ListTagsQueryParamsTagTypeEnum,
  ListTagsQueryParams,
  ListTags200,
  ListTags401,
  ListTagsError,
  ListTagsQueryResponse,
  ListTagsQuery,
  CreateTag201,
  CreateTag401,
  CreateTag409,
  CreateTag422,
  CreateTagError,
  CreateTagMutationRequest,
  CreateTagMutationResponse,
  CreateTagMutation,
  GetTagPathParams,
  GetTagQueryParams,
  GetTag200,
  GetTag401,
  GetTag403,
  GetTag404,
  GetTagError,
  GetTagQueryResponse,
  GetTagQuery,
  DeleteTagPathParams,
  DeleteTag204,
  DeleteTag401,
  DeleteTag403,
  DeleteTag404,
  DeleteTagError,
  DeleteTagMutationResponse,
  DeleteTagMutation,
  UpdateTagPathParams,
  UpdateTag200,
  UpdateTag401,
  UpdateTag403,
  UpdateTag404,
  UpdateTag422,
  UpdateTagError,
  UpdateTagMutationRequest,
  UpdateTagMutationResponse,
  UpdateTagMutation,
} from './types.ts'
export {
  listApiKeys,
  createApiKey,
  getApiKey,
  deleteApiKey,
  updateApiKey,
  getActiveUser,
  listBookmarks,
  createBookmark,
  getBookmark,
  deleteBookmark,
  updateBookmark,
  linkBookmarkTags,
  scrapeBookmark,
  importBookmarks,
  exportBookmarks,
  listCollections,
  createCollection,
  getCollection,
  deleteCollection,
  updateCollection,
  getConfig,
  listFeedEntries,
  getFeedEntry,
  detectFeeds,
  scrapeFeed,
  listStreams,
  createStream,
  getStream,
  deleteStream,
  updateStream,
  listSubscriptions,
  createSubscription,
  getSubscription,
  deleteSubscription,
  updateSubscription,
  linkSubscriptionTags,
  markSubscriptionEntryAsRead,
  markSubscriptionEntryAsUnread,
  importSubscriptions,
  exportSubscriptions,
  listSubscriptionEntries,
  listTags,
  createTag,
  getTag,
  deleteTag,
  updateTag,
} from './client.ts'
export {
  apiErrorCodeEnum,
  bookmarkDateFieldEnum,
  bookmarkTextFieldEnum,
  subscriptionEntryBooleanFieldEnum,
  subscriptionEntryDateFieldEnum,
  subscriptionEntryTextFieldEnum,
  listTagsQueryParamsTagTypeEnum,
} from './types.ts'
export {
  apiErrorSchema,
  apiErrorCodeSchema,
  apiKeySchema,
  apiKeyCreateSchema,
  apiKeyCreatedSchema,
  apiKeyUpdateSchema,
  bookmarkSchema,
  bookmarkCreateSchema,
  bookmarkDateFieldSchema,
  bookmarkDetailsSchema,
  bookmarkFilterSchema,
  bookmarkScrapeSchema,
  bookmarkScrapedSchema,
  bookmarkTextFieldSchema,
  bookmarkUpdateSchema,
  booleanOpSchema,
  collectionSchema,
  collectionCreateSchema,
  collectionUpdateSchema,
  configSchema,
  dateOpSchema,
  feedSchema,
  feedDetectSchema,
  feedDetectedSchema,
  feedEntrySchema,
  feedScrapeSchema,
  linkBookmarkTagsSchema,
  linkSubscriptionTagsSchema,
  oidcConfigSchema,
  paginatedApiKeySchema,
  paginatedBookmarkDetailsSchema,
  paginatedCollectionSchema,
  paginatedFeedEntrySchema,
  paginatedStreamSchema,
  paginatedSubscriptionDetailsSchema,
  paginatedSubscriptionEntryDetailsSchema,
  paginatedTagDetailsSchema,
  storageConfigSchema,
  streamSchema,
  streamCreateSchema,
  streamUpdateSchema,
  subscriptionSchema,
  subscriptionCreateSchema,
  subscriptionDetailsSchema,
  subscriptionEntrySchema,
  subscriptionEntryBooleanFieldSchema,
  subscriptionEntryDateFieldSchema,
  subscriptionEntryDetailsSchema,
  subscriptionEntryFilterSchema,
  subscriptionEntryTextFieldSchema,
  subscriptionUpdateSchema,
  tagSchema,
  tagCreateSchema,
  tagDetailsSchema,
  tagUpdateSchema,
  textOpSchema,
  userSchema,
  listApiKeys200Schema,
  listApiKeys401Schema,
  listApiKeysErrorSchema,
  listApiKeysQueryResponseSchema,
  createApiKey201Schema,
  createApiKey401Schema,
  createApiKey422Schema,
  createApiKeyErrorSchema,
  createApiKeyMutationRequestSchema,
  createApiKeyMutationResponseSchema,
  getApiKeyPathParamsSchema,
  getApiKey200Schema,
  getApiKey401Schema,
  getApiKey403Schema,
  getApiKey404Schema,
  getApiKeyErrorSchema,
  getApiKeyQueryResponseSchema,
  deleteApiKeyPathParamsSchema,
  deleteApiKey204Schema,
  deleteApiKey401Schema,
  deleteApiKey403Schema,
  deleteApiKey404Schema,
  deleteApiKeyErrorSchema,
  deleteApiKeyMutationResponseSchema,
  updateApiKeyPathParamsSchema,
  updateApiKey200Schema,
  updateApiKey401Schema,
  updateApiKey403Schema,
  updateApiKey404Schema,
  updateApiKey422Schema,
  updateApiKeyErrorSchema,
  updateApiKeyMutationRequestSchema,
  updateApiKeyMutationResponseSchema,
  getActiveUser200Schema,
  getActiveUser401Schema,
  getActiveUserErrorSchema,
  getActiveUserQueryResponseSchema,
  listBookmarksQueryParamsSchema,
  listBookmarks200Schema,
  listBookmarks401Schema,
  listBookmarksErrorSchema,
  listBookmarksQueryResponseSchema,
  createBookmark201Schema,
  createBookmark401Schema,
  createBookmark409Schema,
  createBookmark422Schema,
  createBookmarkErrorSchema,
  createBookmarkMutationRequestSchema,
  createBookmarkMutationResponseSchema,
  getBookmarkPathParamsSchema,
  getBookmarkQueryParamsSchema,
  getBookmark200Schema,
  getBookmark401Schema,
  getBookmark403Schema,
  getBookmark404Schema,
  getBookmarkErrorSchema,
  getBookmarkQueryResponseSchema,
  deleteBookmarkPathParamsSchema,
  deleteBookmark204Schema,
  deleteBookmark401Schema,
  deleteBookmark403Schema,
  deleteBookmark404Schema,
  deleteBookmarkErrorSchema,
  deleteBookmarkMutationResponseSchema,
  updateBookmarkPathParamsSchema,
  updateBookmark200Schema,
  updateBookmark401Schema,
  updateBookmark403Schema,
  updateBookmark404Schema,
  updateBookmark422Schema,
  updateBookmarkErrorSchema,
  updateBookmarkMutationRequestSchema,
  updateBookmarkMutationResponseSchema,
  linkBookmarkTagsPathParamsSchema,
  linkBookmarkTags204Schema,
  linkBookmarkTags401Schema,
  linkBookmarkTags403Schema,
  linkBookmarkTags404Schema,
  linkBookmarkTags422Schema,
  linkBookmarkTagsErrorSchema,
  linkBookmarkTagsMutationRequestSchema,
  linkBookmarkTagsMutationResponseSchema,
  scrapeBookmark201Schema,
  scrapeBookmark401Schema,
  scrapeBookmark422Schema,
  scrapeBookmark502Schema,
  scrapeBookmarkErrorSchema,
  scrapeBookmarkMutationRequestSchema,
  scrapeBookmarkMutationResponseSchema,
  importBookmarks204Schema,
  importBookmarks401Schema,
  importBookmarksErrorSchema,
  importBookmarksMutationRequestSchema,
  importBookmarksMutationResponseSchema,
  exportBookmarks200Schema,
  exportBookmarks401Schema,
  exportBookmarksErrorSchema,
  exportBookmarksMutationResponseSchema,
  listCollections200Schema,
  listCollections401Schema,
  listCollectionsErrorSchema,
  listCollectionsQueryResponseSchema,
  createCollection201Schema,
  createCollection401Schema,
  createCollection409Schema,
  createCollection422Schema,
  createCollectionErrorSchema,
  createCollectionMutationRequestSchema,
  createCollectionMutationResponseSchema,
  getCollectionPathParamsSchema,
  getCollection200Schema,
  getCollection401Schema,
  getCollection403Schema,
  getCollection404Schema,
  getCollectionErrorSchema,
  getCollectionQueryResponseSchema,
  deleteCollectionPathParamsSchema,
  deleteCollection204Schema,
  deleteCollection401Schema,
  deleteCollection403Schema,
  deleteCollection404Schema,
  deleteCollectionErrorSchema,
  deleteCollectionMutationResponseSchema,
  updateCollectionPathParamsSchema,
  updateCollection200Schema,
  updateCollection401Schema,
  updateCollection403Schema,
  updateCollection404Schema,
  updateCollection422Schema,
  updateCollectionErrorSchema,
  updateCollectionMutationRequestSchema,
  updateCollectionMutationResponseSchema,
  getConfig200Schema,
  getConfigQueryResponseSchema,
  listFeedEntriesQueryParamsSchema,
  listFeedEntries200Schema,
  listFeedEntries401Schema,
  listFeedEntriesErrorSchema,
  listFeedEntriesQueryResponseSchema,
  getFeedEntryPathParamsSchema,
  getFeedEntry200Schema,
  getFeedEntry401Schema,
  getFeedEntry403Schema,
  getFeedEntry404Schema,
  getFeedEntryErrorSchema,
  getFeedEntryQueryResponseSchema,
  detectFeeds201Schema,
  detectFeeds401Schema,
  detectFeeds422Schema,
  detectFeeds502Schema,
  detectFeedsErrorSchema,
  detectFeedsMutationRequestSchema,
  detectFeedsMutationResponseSchema,
  scrapeFeed201Schema,
  scrapeFeed401Schema,
  scrapeFeed422Schema,
  scrapeFeed502Schema,
  scrapeFeedErrorSchema,
  scrapeFeedMutationRequestSchema,
  scrapeFeedMutationResponseSchema,
  listStreams200Schema,
  listStreams401Schema,
  listStreamsErrorSchema,
  listStreamsQueryResponseSchema,
  createStream201Schema,
  createStream401Schema,
  createStream409Schema,
  createStream422Schema,
  createStreamErrorSchema,
  createStreamMutationRequestSchema,
  createStreamMutationResponseSchema,
  getStreamPathParamsSchema,
  getStream200Schema,
  getStream401Schema,
  getStream403Schema,
  getStream404Schema,
  getStreamErrorSchema,
  getStreamQueryResponseSchema,
  deleteStreamPathParamsSchema,
  deleteStream204Schema,
  deleteStream401Schema,
  deleteStream403Schema,
  deleteStream404Schema,
  deleteStreamErrorSchema,
  deleteStreamMutationResponseSchema,
  updateStreamPathParamsSchema,
  updateStream200Schema,
  updateStream401Schema,
  updateStream403Schema,
  updateStream404Schema,
  updateStream422Schema,
  updateStreamErrorSchema,
  updateStreamMutationRequestSchema,
  updateStreamMutationResponseSchema,
  listSubscriptionsQueryParamsSchema,
  listSubscriptions200Schema,
  listSubscriptions401Schema,
  listSubscriptionsErrorSchema,
  listSubscriptionsQueryResponseSchema,
  createSubscription201Schema,
  createSubscription401Schema,
  createSubscription409Schema,
  createSubscription422Schema,
  createSubscriptionErrorSchema,
  createSubscriptionMutationRequestSchema,
  createSubscriptionMutationResponseSchema,
  getSubscriptionPathParamsSchema,
  getSubscriptionQueryParamsSchema,
  getSubscription200Schema,
  getSubscription401Schema,
  getSubscription403Schema,
  getSubscription404Schema,
  getSubscriptionErrorSchema,
  getSubscriptionQueryResponseSchema,
  deleteSubscriptionPathParamsSchema,
  deleteSubscription204Schema,
  deleteSubscription401Schema,
  deleteSubscription403Schema,
  deleteSubscription404Schema,
  deleteSubscriptionErrorSchema,
  deleteSubscriptionMutationResponseSchema,
  updateSubscriptionPathParamsSchema,
  updateSubscription200Schema,
  updateSubscription401Schema,
  updateSubscription403Schema,
  updateSubscription404Schema,
  updateSubscription422Schema,
  updateSubscriptionErrorSchema,
  updateSubscriptionMutationRequestSchema,
  updateSubscriptionMutationResponseSchema,
  linkSubscriptionTagsPathParamsSchema,
  linkSubscriptionTags204Schema,
  linkSubscriptionTags401Schema,
  linkSubscriptionTags403Schema,
  linkSubscriptionTags404Schema,
  linkSubscriptionTags422Schema,
  linkSubscriptionTagsErrorSchema,
  linkSubscriptionTagsMutationRequestSchema,
  linkSubscriptionTagsMutationResponseSchema,
  markSubscriptionEntryAsReadPathParamsSchema,
  markSubscriptionEntryAsRead200Schema,
  markSubscriptionEntryAsRead401Schema,
  markSubscriptionEntryAsRead403Schema,
  markSubscriptionEntryAsRead404Schema,
  markSubscriptionEntryAsRead422Schema,
  markSubscriptionEntryAsReadErrorSchema,
  markSubscriptionEntryAsReadMutationResponseSchema,
  markSubscriptionEntryAsUnreadPathParamsSchema,
  markSubscriptionEntryAsUnread200Schema,
  markSubscriptionEntryAsUnread401Schema,
  markSubscriptionEntryAsUnread403Schema,
  markSubscriptionEntryAsUnread404Schema,
  markSubscriptionEntryAsUnread422Schema,
  markSubscriptionEntryAsUnreadErrorSchema,
  markSubscriptionEntryAsUnreadMutationResponseSchema,
  importSubscriptions204Schema,
  importSubscriptions401Schema,
  importSubscriptionsErrorSchema,
  importSubscriptionsMutationRequestSchema,
  importSubscriptionsMutationResponseSchema,
  exportSubscriptions200Schema,
  exportSubscriptions401Schema,
  exportSubscriptionsErrorSchema,
  exportSubscriptionsMutationResponseSchema,
  listSubscriptionEntriesQueryParamsSchema,
  listSubscriptionEntries200Schema,
  listSubscriptionEntries401Schema,
  listSubscriptionEntriesErrorSchema,
  listSubscriptionEntriesQueryResponseSchema,
  listTagsQueryParamsSchema,
  listTags200Schema,
  listTags401Schema,
  listTagsErrorSchema,
  listTagsQueryResponseSchema,
  createTag201Schema,
  createTag401Schema,
  createTag409Schema,
  createTag422Schema,
  createTagErrorSchema,
  createTagMutationRequestSchema,
  createTagMutationResponseSchema,
  getTagPathParamsSchema,
  getTagQueryParamsSchema,
  getTag200Schema,
  getTag401Schema,
  getTag403Schema,
  getTag404Schema,
  getTagErrorSchema,
  getTagQueryResponseSchema,
  deleteTagPathParamsSchema,
  deleteTag204Schema,
  deleteTag401Schema,
  deleteTag403Schema,
  deleteTag404Schema,
  deleteTagErrorSchema,
  deleteTagMutationResponseSchema,
  updateTagPathParamsSchema,
  updateTag200Schema,
  updateTag401Schema,
  updateTag403Schema,
  updateTag404Schema,
  updateTag422Schema,
  updateTagErrorSchema,
  updateTagMutationRequestSchema,
  updateTagMutationResponseSchema,
} from './zod.ts'
