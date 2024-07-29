use sea_orm_migration::{prelude::ConnectionTrait, sea_orm::ExecResult, DbErr, SchemaManager};

pub async fn create_updated_at_fn<'a>(manager: &'a SchemaManager<'a>) -> Result<ExecResult, DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            "
      CREATE OR REPLACE FUNCTION handle_updated_at() RETURNS trigger AS $$
       BEGIN
             NEW.updated_at = CURRENT_TIMESTAMP;
             RETURN NEW;
         END;
$$ LANGUAGE plpgsql",
        )
        .await
}

pub async fn create_updated_at_trigger<'a>(
    manager: &'a SchemaManager<'a>,
    table_name: &str,
) -> Result<ExecResult, DbErr> {
    manager
        .get_connection()
        .execute_unprepared(&format!(
            "
 CREATE TRIGGER {table}_updated_at
 BEFORE UPDATE ON \"{table}\"
    FOR EACH ROW
EXECUTE FUNCTION handle_updated_at()",
            table = table_name,
        ))
        .await
}
