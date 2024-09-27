use media::MediaGroup;

pub mod media;

#[derive(Debug, Clone, Default)]
pub struct Extension {
    pub media_group: Option<MediaGroup>,
}
