use sea_orm_migration::{prelude::ConnectionTrait, sea_orm::ExecResult, DbErr, SchemaManager};

pub async fn create_updated_at_trigger<'a>(
    manager: &'a SchemaManager<'a>,
    table: String,
    columns: Vec<String>,
) -> Result<ExecResult, DbErr> {
    let column_changes = columns
        .iter()
        .map(|e| format!("OLD.{0} IS NOT NEW.{0}", e))
        .collect::<Vec<_>>()
        .join(" OR ");

    let query = format!(
        "
CREATE TRIGGER IF NOT EXISTS {table}_updated_at
 AFTER UPDATE ON {table}
   FOR EACH ROW
  WHEN (
         OLD.id = NEW.id AND (
           {column_changes}
         )
       )
 BEGIN
       UPDATE {table}
          SET updated_at = CURRENT_TIMESTAMP
        WHERE id = NEW.id;
   END;
",
        table = table,
        column_changes = column_changes
    );

    manager.get_connection().execute_unprepared(&query).await
}
