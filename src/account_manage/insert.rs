use crate::{
    preludes::rocket_prelude::*,
    subbranch_manage::{query_subbranch, set_subbranch_asset},
    utility::GenericError,
};
use chrono::prelude::*;
use sqlx::types::BigDecimal;

#[derive(Debug, FromForm, Default, Serialize)]
pub struct AccountSubmit {
    pub clientIDs: String,
    pub accountType: String,
    pub balance: String,
    pub currencyType: String,
    pub subbranchName: String,
    pub overdraft: String,
    pub interest: String,
}

/// Add account entity, owning relation and update the assets of subbranch
pub async fn add_new_account_and_own(
    db: &mut Connection<BankManage>,
    submission: &AccountSubmit,
) -> Result<String, GenericError> {
    let account_id = add_account_entity(db, submission).await?;
    let clientIDs: Vec<_> = crate::utility::get_list_from_input(&submission.clientIDs);
    for client_id in clientIDs {
        add_owning_relation(
            db,
            client_id,
            account_id.clone(),
            submission.accountType.clone(),
            submission.subbranchName.clone(),
        )
        .await?;
    }
    let subbranch_asset = query_subbranch(db, &submission.subbranchName)
        .await?
        .subbranchAsset;
    let new_subbranch_asset = subbranch_asset + submission.balance.parse::<BigDecimal>()?;
    set_subbranch_asset(db, &submission.subbranchName, &new_subbranch_asset).await?;

    Ok(account_id)
}

/// Add entity to `account`, `savingaccount`/`checkingaccount`
async fn add_account_entity(
    db: &mut Connection<BankManage>,
    submission: &AccountSubmit,
) -> Result<String, GenericError> {
    let account_id = uuid::Uuid::new_v4().to_string();
    let cur_date = Local::now().format("%Y-%m-%d").to_string();

    // into table `account`
    sqlx::query("insert into account(accountID, balance, openDate) values (?, ?, ?)")
        .bind(&account_id)
        .bind(&submission.balance)
        .bind(&cur_date)
        .execute(&mut **db)
        .await?;

    match &submission.accountType as &str {
        "savingAccount" => {
            // into table `savingaccount`
            let interest = submission.interest.parse::<f64>()?;
            let currency_type = submission.currencyType.clone();
            sqlx::query("insert into savingaccount(accountID, balance, openDate, interest, currencyType) values (?, ?, ?, ?, ?)")
                .bind(&account_id)
                .bind(&submission.balance)
                .bind(&cur_date)
                .bind(&interest)
                .bind(&currency_type)
                .execute(&mut **db).await?;
        }
        "checkingAccount" => {
            // into table `checkingaccount`
            let overdraft = submission.overdraft.parse::<f64>()?;
            sqlx::query("insert into checkingaccount(accountID, balance, openDate, overdraft) values (?, ?, ?, ?)")
            .bind(&account_id)
            .bind(&submission.balance)
            .bind(&cur_date)
            .bind(&overdraft)
            .execute(&mut **db).await?;
        }
        _ => {
            return Err(Box::new(AccountConstraintError {}));
        }
    }
    Ok(account_id)
}

#[derive(Debug)]
pub struct AccountConstraintError {}

impl std::fmt::Display for AccountConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "account constraint error")
    }
}

impl std::error::Error for AccountConstraintError {}

/// Add relationship between client and account. Add entity to `own`, `accountmanagement`.
pub async fn add_owning_relation(
    db: &mut Connection<BankManage>,
    client_id: String,
    account_id: String,
    account_type: String,
    subbranchName: String,
) -> Result<(), GenericError> {
    let cur_time = Local::now().format("%Y-%m-%d %T").to_string();

    // into table `own`
    sqlx::query("insert into own(accountID, clientID, lastVisitTime) values (?, ?, ?)")
        .bind(&account_id)
        .bind(&client_id)
        .bind(cur_time)
        .execute(&mut **db)
        .await?;

    // filter in table `accountmanagement`
    let account_manage_entry = sqlx::query_as!(
        AccountManagement,
        "SELECT * FROM accountmanagement WHERE subbranchName=? and clientID=?",
        subbranchName,
        client_id
    )
    .fetch_one(&mut **db)
    .await;

    macro_rules! add_to_accountmanagement {
        ($(($account_type: ident, $attr_name: ident)),+) => {
            match  &account_type as &str{
            $(
                stringify!($account_type) => {
                    // into table `accountmanagement`
                    match account_manage_entry {
                        Err(sqlx::Error::RowNotFound) => {
                            let account_manage_entry = AccountManagement {
                                subbranchName: subbranchName.clone(),
                                clientID: client_id.clone(),
                                $attr_name: Some(account_id.clone()),
                                ..AccountManagement::default()
                            };
                            sqlx::query("INSERT INTO accountmanagement (subbranchName,clientID,savingAccountID,checkingAccountID)VALUES(?, ?, ?, ?)")
                            .bind(account_manage_entry.subbranchName)
                            .bind(account_manage_entry.clientID)
                            .bind(account_manage_entry.savingAccountID)
                            .bind(account_manage_entry.checkingAccountID)
                            .execute(&mut **db).await?;
                        }
                        Err(e)=> {return Err(Box::new(e));}
                        Ok(mut account_manage_entry) => {
                            if !account_manage_entry.$attr_name.is_none() {
                                return Err(Box::new(AccountConstraintError {}));
                            } else {
                                account_manage_entry.$attr_name = Some(account_id.clone());
                                sqlx::query("UPDATE accountmanagement SET
                                savingAccountID=?,
                                checkingAccountID=?
                                WHERE subbranchName=? and clientID=? 
                                ")
                                .bind(account_manage_entry.savingAccountID)
                                .bind(account_manage_entry.checkingAccountID)
                                .bind(account_manage_entry.subbranchName)
                                .bind(account_manage_entry.clientID)
                                .execute(&mut **db).await?;
                            }
                        }
                    }
                }
            )+
            _ => {
            return Err(Box::new(AccountConstraintError {}));
        }
            }
        };
    }

    add_to_accountmanagement!(
        (savingAccount, savingAccountID),
        (checkingAccount, checkingAccountID)
    );
    Ok(())
}
