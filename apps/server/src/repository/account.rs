use colette_core::{
    Account,
    account::{AccountCreateData, AccountFindParams, AccountRepository, Error},
};
use colette_model::{AccountRow, accounts, users};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, FromQueryResult, TransactionTrait,
    sea_query::{Expr, Query},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteAccountRepository {
    db: DatabaseConnection,
}

impl SqliteAccountRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_account(&self, params: AccountFindParams) -> Result<Account, Error> {
        let query = Query::select()
            .column((users::Entity, users::Column::Email))
            .columns([
                (accounts::Entity, accounts::Column::ProviderId),
                (accounts::Entity, accounts::Column::AccountId),
                (accounts::Entity, accounts::Column::PasswordHash),
                (accounts::Entity, accounts::Column::UserId),
            ])
            .from(accounts::Entity)
            .inner_join(
                users::Entity,
                Expr::col((users::Entity, users::Column::Id))
                    .eq(Expr::col((accounts::Entity, accounts::Column::UserId))),
            )
            .and_where(
                Expr::col((accounts::Entity, accounts::Column::ProviderId)).eq(params.provider_id),
            )
            .and_where(
                Expr::col((accounts::Entity, accounts::Column::AccountId))
                    .eq(params.account_id.as_str()),
            )
            .to_owned();

        let Some(account) =
            AccountRow::find_by_statement(self.db.get_database_backend().build(&query))
                .one(&self.db)
                .await?
        else {
            return Err(Error::NotFound(params.account_id));
        };

        Ok(account.into())
    }

    async fn create_account(&self, data: AccountCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let user_id = Uuid::new_v4();

        let query = Query::insert()
            .into_table(users::Entity)
            .columns([
                users::Column::Id,
                users::Column::Email,
                users::Column::DisplayName,
            ])
            .values_panic([
                user_id.to_string().into(),
                data.email.into(),
                data.display_name.into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        let query = Query::insert()
            .into_table(accounts::Entity)
            .columns([
                accounts::Column::ProviderId,
                accounts::Column::AccountId,
                accounts::Column::PasswordHash,
                accounts::Column::UserId,
            ])
            .values_panic([
                data.provider_id.into(),
                data.account_id.into(),
                data.password_hash.into(),
                user_id.to_string().into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        tx.commit().await?;

        Ok(user_id)
    }
}
