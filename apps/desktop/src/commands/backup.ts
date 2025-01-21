import type { BackupAPI } from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class BackupCommands implements BackupAPI {
  async import(data: File): Promise<void> {
    return invoke('import_opml', {
      data: new Uint8Array(await data.arrayBuffer()),
    }).then()
  }
}
