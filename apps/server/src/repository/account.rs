use colette_core::{
    Account,
    account::{AccountCreateData, AccountFindParams, AccountRepository, Error},
};
use colette_model::{AccountWithUser, accounts, users};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, TransactionTrait};
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
        let Some((account, Some(user))) =
            accounts::Entity::find_by_id((params.provider_id, params.account_id.clone()))
                .find_also_related(users::Entity)
                .one(&self.db)
                .await?
        else {
            return Err(Error::NotFound(params.account_id));
        };

        Ok(AccountWithUser { account, user }.into())
    }

    async fn create_account(&self, data: AccountCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let user_id = Uuid::new_v4();
        let user_model = users::ActiveModel {
            id: ActiveValue::Set(user_id.into()),
            email: ActiveValue::Set(data.email),
            display_name: ActiveValue::Set(data.display_name),
            ..Default::default()
        };
        user_model.insert(&tx).await?;

        let accout_model = accounts::ActiveModel {
            provider_id: ActiveValue::Set(data.provider_id),
            account_id: ActiveValue::Set(data.account_id),
            password_hash: ActiveValue::Set(data.password_hash),
            user_id: ActiveValue::Set(user_id.into()),
            ..Default::default()
        };
        accout_model.insert(&tx).await?;

        tx.commit().await?;

        Ok(user_id)
    }
}
