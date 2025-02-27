use colette_core::{
    Account,
    accounts::{AccountCreateData, AccountFindParams, AccountRepository, Error},
    common::{Creatable, Findable},
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, TransactionTrait};
use uuid::Uuid;

use super::entity;

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
impl Findable for SqliteAccountRepository {
    type Params = AccountFindParams;
    type Output = Result<Account, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let Some((account, Some(user))) =
            entity::accounts::Entity::find_by_id((params.provider_id, params.account_id.clone()))
                .find_also_related(entity::users::Entity)
                .one(&self.db)
                .await?
        else {
            return Err(Error::NotFound(params.account_id));
        };

        Ok(AccountWithUser { account, user }.into())
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteAccountRepository {
    type Data = AccountCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let user_id = Uuid::new_v4();
        let user = entity::users::ActiveModel {
            id: ActiveValue::Set(user_id.into()),
            email: ActiveValue::Set(data.email),
            display_name: ActiveValue::Set(data.display_name),
            ..Default::default()
        };
        user.insert(&tx).await?;

        let account = entity::accounts::ActiveModel {
            provider_id: ActiveValue::Set(data.provider_id),
            account_id: ActiveValue::Set(data.account_id),
            password_hash: ActiveValue::Set(data.password_hash),
            user_id: ActiveValue::Set(user_id.into()),
            ..Default::default()
        };
        account.insert(&tx).await?;

        tx.commit().await?;

        Ok(user_id)
    }
}

impl AccountRepository for SqliteAccountRepository {}

struct AccountWithUser {
    account: entity::accounts::Model,
    user: entity::users::Model,
}

impl From<AccountWithUser> for Account {
    fn from(value: AccountWithUser) -> Self {
        Self {
            id: value.user.id.parse().unwrap(),
            email: value.user.email,
            provider_id: value.account.provider_id,
            account_id: value.account.account_id,
            password_hash: value.account.password_hash,
        }
    }
}
