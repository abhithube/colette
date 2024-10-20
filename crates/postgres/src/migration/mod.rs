use sqlx::migrate::{Migration, MigrationType};

mod V0001__initial_user;
mod V0002__initial_feed;
mod V0003__initial_bookmark;
mod V0004__initial_profile_feed;
mod V0005__initial_profile_bookmark;
mod V0006__initial_tag;
mod V0007__initial_smart_feed;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            1,
            "initial_user".into(),
            MigrationType::Simple,
            V0001__initial_user::migration().into(),
            false,
        ),
        Migration::new(
            2,
            "initial_feed".into(),
            MigrationType::Simple,
            V0002__initial_feed::migration().into(),
            false,
        ),
        Migration::new(
            3,
            "initial_bookmark".into(),
            MigrationType::Simple,
            V0003__initial_bookmark::migration().into(),
            false,
        ),
        Migration::new(
            4,
            "initial_profile_feed".into(),
            MigrationType::Simple,
            V0004__initial_profile_feed::migration().into(),
            false,
        ),
        Migration::new(
            5,
            "initial_profile_bookmark".into(),
            MigrationType::Simple,
            V0005__initial_profile_bookmark::migration().into(),
            false,
        ),
        Migration::new(
            6,
            "initial_tag".into(),
            MigrationType::Simple,
            V0006__initial_tag::migration().into(),
            false,
        ),
        Migration::new(
            7,
            "initial_smart_feed".into(),
            MigrationType::Simple,
            V0007__initial_smart_feed::migration().into(),
            false,
        ),
    ]
}
