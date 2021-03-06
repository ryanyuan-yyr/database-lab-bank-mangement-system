use std::collections::HashMap;

use sqlx::Executor;

use super::preludes::rocket_prelude::*;
use crate::{
    account_manage::{delete::*, update::*},
    commit, error_template, rollback, start_transaction,
    utility::{get_list_from_input, get_restriction, Restriction},
};

#[derive(Serialize)]
struct EditSavingAccountContext {
    id: String,
    clientIDs: String,
    balance: String,
    currencyType: String,
    interest: String,
    restriction: Restriction,
}

#[derive(Serialize)]
struct EditCheckingAccountContext {
    id: String,
    clientIDs: String,
    balance: String,
    overdraft: String,
    restriction: Restriction,
}

#[get("/edit/account?<id>")]
pub async fn get_edit_account(mut db: Connection<BankManage>, id: String) -> Template {
    use super::account_manage::query::*;
    let clients = match query_associated_clients(&mut db, id.clone()).await {
        Err(e) => return error_template!(e, "Error fetching account info"),
        Ok(result) => result
            .into_iter()
            .fold(String::new(), |joined, cur| joined + " " + &cur),
    };
    match query_account_by_id(&mut db, &id).await {
        Err(e) => return error_template!(e, "Error fetching account info"),
        Ok((specific_account, _)) => match specific_account {
            SpecificAccount::SavingAccount(saving_account) => Template::render(
                "edit-saving-account",
                EditSavingAccountContext {
                    id: saving_account.accountID,
                    clientIDs: clients,
                    balance: saving_account.balance.to_string(),
                    currencyType: saving_account.currencyType,
                    interest: saving_account.interest.to_string(),
                    restriction: get_restriction(),
                },
            ),
            SpecificAccount::CheckingAccount(checking_account) => Template::render(
                "edit-checking-account",
                EditCheckingAccountContext {
                    id: checking_account.accountID,
                    clientIDs: clients,
                    balance: checking_account.balance.to_string(),
                    overdraft: checking_account.overdraft.to_string(),
                    restriction: get_restriction(),
                },
            ),
        },
    }
}

#[post("/edit/account/saving?<id>", data = "<form>")]
pub async fn act_edit_saving_account(
    mut db: Connection<BankManage>,
    id: String,
    form: Form<Contextual<'_, SavingAccountSubmit>>,
) -> (Status, Template) {
    if form.value.is_none() {
        return (
            form.context.status(),
            error_template!("Error receiving form"),
        );
    }
    let submission = form.value.as_ref().unwrap();
    start_transaction!(db);
    let updated_associated_client_IDs: std::collections::HashSet<String> =
        get_list_from_input(&submission.clientIDs);
    match update_saving_account_and_own(
        &mut db,
        id.clone(),
        submission.clone(),
        updated_associated_client_IDs,
    )
    .await
    {
        Ok(_) => {
            commit!(db);
            (
                form.context.status(),
                Template::render("update-account-success", HashMap::from([("id", id)])),
            )
        }
        Err(e) => {
            rollback!(db);
            (
                form.context.status(),
                error_template!(e, "Error updating account"),
            )
        }
    }
}

#[post("/edit/account/checking?<id>", data = "<form>")]
pub async fn act_edit_checking_account(
    mut db: Connection<BankManage>,
    id: String,
    form: Form<Contextual<'_, CheckingAccountSubmit>>,
) -> (Status, Template) {
    if form.value.is_none() {
        return (
            form.context.status(),
            error_template!("Error receiving form"),
        );
    }
    let submission = form.value.as_ref().unwrap();
    start_transaction!(db);
    let updated_associated_client_IDs: std::collections::HashSet<String> =
        get_list_from_input(&submission.clientIDs);
    match update_checking_account_and_own(
        &mut db,
        id.clone(),
        submission.clone(),
        updated_associated_client_IDs,
    )
    .await
    {
        Ok(_) => {
            commit!(db);
            (
                form.context.status(),
                Template::render("update-account-success", HashMap::from([("id", id)])),
            )
        }
        Err(e) => {
            rollback!(db);
            (
                form.context.status(),
                error_template!(e, "Error updating account"),
            )
        }
    }
}

#[get("/delete/account?<id>")]
pub async fn delete_account(mut db: Connection<BankManage>, id: String) -> Template {
    start_transaction!(db);
    let template = match delete_account_and_own(&mut db, id).await {
        Ok(_) => {
            commit!(db);
            Template::render("delete-account-success", &Context::default())
        }
        Err(e) => {
            rollback!(db);
            error_template!(e, "Error deleting account")
        }
    };

    template
}
