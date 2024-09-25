import {
  Profile,
  type ProfileAPI,
  ProfileCreate,
  ProfileList,
  ProfileUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class ProfileCommands implements ProfileAPI {
  list(): Promise<ProfileList> {
    return invoke('list_profiles').then(ProfileList.parse)
  }

  get(id: string): Promise<Profile> {
    return invoke('get_profile', { id }).then(Profile.parse)
  }

  getActive(): Promise<Profile> {
    return invoke('get_active_profile').then(Profile.parse)
  }

  create(data: ProfileCreate): Promise<Profile> {
    return invoke('create_profile', { data: ProfileCreate.parse(data) }).then(
      Profile.parse,
    )
  }

  update(id: string, data: ProfileUpdate): Promise<Profile> {
    return invoke('update_profile', {
      id,
      data: ProfileUpdate.parse(data),
    }).then(Profile.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_profile', { id }).then()
  }
}
