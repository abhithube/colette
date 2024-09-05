import type { ApiClient } from './openapi.gen'

export interface BackupAPI {
  import(data: File): Promise<void>
}

export class HTTPBackupAPI implements BackupAPI {
  constructor(private client: ApiClient) {}

  async import(data: File): Promise<void> {
    return this.client
      .post('/backups/opml/import', {
        body: await Array.fromAsync(fileToAsyncIterator(data)),
      })
      .then()
  }
}

async function* fileToAsyncIterator(file: File) {
  const buffer = await file.arrayBuffer()

  for (const chunk of new Uint8Array(buffer)) {
    yield chunk
  }
}
