use std::borrow::Cow;

use refinery_core::Migration;

pub struct MigrationFile {
    path: Cow<'static, str>,
    contents: Cow<'static, [u8]>,
}

impl MigrationFile {
    pub fn new(path: Cow<'static, str>, contents: Cow<'static, [u8]>) -> Self {
        Self { path, contents }
    }
}

pub fn load_migrations(
    files: &mut Vec<MigrationFile>,
) -> Result<Vec<Migration>, refinery_core::Error> {
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut counter = 1;
    let mut migrations = Vec::<Migration>::new();

    for file in files {
        if !file.path.ends_with(".sql") {
            continue;
        }

        let parts = file.path.split("_").collect::<Vec<_>>();
        let Some(name) = parts.get(1) else {
            continue;
        };

        let migration = Migration::unapplied(
            &format!("V{}__{}", counter, name),
            &String::from_utf8_lossy(&file.contents),
        )?;
        migrations.push(migration);

        counter += 1;
    }

    Ok(migrations)
}
