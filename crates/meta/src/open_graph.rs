use crate::Metadata;

#[derive(Debug, Clone, Default)]
pub struct OpenGraph {
    pub title: String,
    pub r#type: Type,
    pub images: Vec<Image>,
    pub url: String,
    pub audios: Vec<Audio>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub videos: Vec<Video>,
}

#[derive(Debug, Clone, Default)]
pub enum Type {
    // MusicSong,
    // MusicAlbum,
    // MusicPlaylist,
    // MusicRadioStation,
    // VideoMovie,
    // VideoEpisode,
    // VideoTvShow,
    // VideoOther,
    Article(Article),
    // Book,
    // Profile,
    #[default]
    Website,
}

#[derive(Debug, Clone, Default)]
pub struct Article {
    pub published_time: Option<String>,
    pub modified_time: Option<String>,
    pub expiration_time: Option<String>,
    // pub author,
    pub section: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Image {
    pub url: String,
    pub secure_url: Option<String>,
    pub r#type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Audio {
    pub url: String,
    pub secure_url: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Video {
    pub url: String,
    pub secure_url: Option<String>,
    pub r#type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl Metadata {
    pub(crate) fn handle_open_graph(&mut self, property: &str, content: String) {
        let open_graph = self.open_graph.get_or_insert_with(OpenGraph::default);

        match property {
            "og:title" => {
                open_graph.title = content;
            }
            "og:type" => match content.as_str() {
                "article" => open_graph.r#type = Type::Article(Article::default()),
                _ => open_graph.r#type = Type::Website,
            },
            "og:url" => {
                open_graph.url = content;
            }
            "og:description" => {
                open_graph.description = Some(content);
            }
            "og:site_name" => {
                open_graph.site_name = Some(content);
            }
            "og:image" => {
                open_graph.images.push(Image {
                    url: content,
                    ..Default::default()
                });
            }
            "og:image:secure_url" => {
                if let Some(image) = open_graph.images.first_mut() {
                    image.secure_url = Some(content);
                }
            }
            "og:image:type" => {
                if let Some(image) = open_graph.images.first_mut() {
                    image.r#type = Some(content);
                }
            }
            "og:image:width" => {
                if let Some(image) = open_graph.images.first_mut() {
                    image.width = content.parse().ok();
                }
            }
            "og:image:height" => {
                if let Some(image) = open_graph.images.first_mut() {
                    image.height = content.parse().ok();
                }
            }
            "og:image:alt" => {
                if let Some(image) = open_graph.images.first_mut() {
                    image.alt = Some(content);
                }
            }
            "og:audio" => {
                open_graph.audios.push(Audio {
                    url: content,
                    ..Default::default()
                });
            }
            "og:audio:secure_url" => {
                if let Some(audio) = open_graph.audios.first_mut() {
                    audio.secure_url = Some(content);
                }
            }
            "og:audio:type" => {
                if let Some(audio) = open_graph.audios.first_mut() {
                    audio.r#type = Some(content);
                }
            }
            "og:video" => {
                open_graph.videos.push(Video {
                    url: content,
                    ..Default::default()
                });
            }
            "og:video:secure_url" => {
                if let Some(video) = open_graph.videos.first_mut() {
                    video.secure_url = Some(content);
                }
            }
            "og:video:type" => {
                if let Some(video) = open_graph.videos.first_mut() {
                    video.r#type = Some(content);
                }
            }
            "og:video:width" => {
                if let Some(video) = open_graph.videos.first_mut() {
                    video.width = content.parse().ok();
                }
            }
            "og:video:height" => {
                if let Some(video) = open_graph.videos.first_mut() {
                    video.height = content.parse().ok();
                }
            }
            "article:published_time" => {
                if let Type::Article(ref mut article) = open_graph.r#type {
                    article.published_time = Some(content);
                }
            }
            "article:modified_time" => {
                if let Type::Article(ref mut article) = open_graph.r#type {
                    article.modified_time = Some(content);
                }
            }
            "article:expiration_time" => {
                if let Type::Article(ref mut article) = open_graph.r#type {
                    article.expiration_time = Some(content);
                }
            }
            "article:section" => {
                if let Type::Article(ref mut article) = open_graph.r#type {
                    article.section = Some(content);
                }
            }
            "article:tags" => {
                if let Type::Article(ref mut article) = open_graph.r#type {
                    article.tags.push(content);
                }
            }
            _ => {}
        }
    }
}
