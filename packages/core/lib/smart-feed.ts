// import {
//   type ApiClient,
//   Paginated_SmartFeed,
//   SmartFeed,
//   SmartFeedCreate,
//   SmartFeedUpdate,
// } from './openapi.gen'

// export type SmartFeedList = Paginated_SmartFeed
// export const SmartFeedList = Paginated_SmartFeed

// export interface SmartFeedAPI {
//   list(): Promise<SmartFeedList>

//   get(id: string): Promise<SmartFeed>

//   create(data: SmartFeedCreate): Promise<SmartFeed>

//   update(id: string, data: SmartFeedUpdate): Promise<SmartFeed>

//   delete(id: string): Promise<void>
// }

// export class HTTPSmartFeedAPI implements SmartFeedAPI {
//   constructor(private client: ApiClient) {}

//   list(): Promise<SmartFeedList> {
//     return this.client.get('/smartFeeds').then(SmartFeedList.parse)
//   }

//   get(id: string): Promise<SmartFeed> {
//     return this.client
//       .get('/smartFeeds/{id}', {
//         path: {
//           id,
//         },
//       })
//       .then(SmartFeed.parse)
//   }

//   create(data: SmartFeedCreate): Promise<SmartFeed> {
//     return this.client
//       .post('/smartFeeds', {
//         body: SmartFeedCreate.parse(data),
//       })
//       .then(SmartFeed.parse)
//   }

//   update(id: string, data: SmartFeedUpdate): Promise<SmartFeed> {
//     return this.client
//       .patch('/smartFeeds/{id}', {
//         path: {
//           id,
//         },
//         body: SmartFeedUpdate.parse(data),
//       })
//       .then(SmartFeed.parse)
//   }

//   delete(id: string): Promise<void> {
//     return this.client
//       .delete('/smartFeeds/{id}', {
//         path: {
//           id,
//         },
//       })
//       .then()
//   }
// }
