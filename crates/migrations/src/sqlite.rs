use sea_orm_migration::{prelude::ConnectionTrait, sea_orm::ExecResult, DbErr, SchemaManager};

pub async fn create_updated_at_trigger<'a>(
    manager: &'a SchemaManager<'a>,
    table_name: &str,
) -> Result<ExecResult, DbErr> {
    manager
        .get_connection()
        .execute_unprepared(&format!(
            "
CREATE TRIGGER {table}_updated_at
 AFTER UPDATE ON {table}
   FOR EACH ROW
 BEGIN
       UPDATE {table}
          SET updated_at = CURRENT_TIMESTAMP
        WHERE id = new.id;
   END",
            table = table_name,
        ))
        .await
}
