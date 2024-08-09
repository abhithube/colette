use sea_orm_migration::{prelude::ConnectionTrait, sea_orm::ExecResult, DbErr, SchemaManager};

pub async fn create_updated_at_fn<'a>(manager: &'a SchemaManager<'a>) -> Result<ExecResult, DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            "
     CREATE OR REPLACE FUNCTION handle_updated_at() RETURNS trigger AS $$
      BEGIN
            IF (OLD.* IS DISTINCT FROM NEW.*) THEN
                NEW.updated_at = CURRENT_TIMESTAMP;
            ELSE
                NEW.updated_at = OLD.updated_at;
            END IF;
            RETURN NEW;
        END;
$$ LANGUAGE plpgsql",
        )
        .await
}

pub async fn create_updated_at_trigger<'a>(
    manager: &'a SchemaManager<'a>,
    table: &str,
) -> Result<ExecResult, DbErr> {
    manager
        .get_connection()
        .execute_unprepared(&format!(
            "
 CREATE OR REPLACE TRIGGER \"{table}_updated_at\"
 BEFORE UPDATE ON \"{table}\"
    FOR EACH ROW
EXECUTE FUNCTION handle_updated_at()",
            table = table,
        ))
        .await
}
