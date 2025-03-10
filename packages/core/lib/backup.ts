import type { ApiClient } from './openapi.gen'

export interface BackupAPI {
  importOPML(data: File): Promise<void>
}

export class HTTPBackupAPI implements BackupAPI {
  constructor(private client: ApiClient) {}

  async importOPML(data: File): Promise<void> {
    return this.client
      .post('/backups/opml/import', {
        body: new Uint8Array(await data.arrayBuffer()),
        header: {
          'Content-Type': 'application/octet-stream',
        },
      } as any)
      .then()
  }
}
