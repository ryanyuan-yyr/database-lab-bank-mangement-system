use chrono::NaiveDate;
use serde::Serialize;

#[derive(Default, Serialize, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct Client {
    pub clientID: String,
    pub employeeID: Option<String>,
    pub clientName: Option<String>,
    pub clientTel: Option<String>,
    pub clientAddr: Option<String>,
    pub contactName: Option<String>,
    pub contactTel: Option<String>,
    pub contactEmail: Option<String>,
    pub contactRelationship: Option<String>,
    pub serviceType: Option<String>,
}

#[derive(Default, Serialize, PartialEq, Eq, Hash)]
pub struct AccountManagement {
    pub subbranchName: String,
    pub clientID: String,
    pub savingAccountID: Option<String>,
    pub checkingAccountID: Option<String>,
}

#[derive(PartialEq, sqlx::FromRow, Debug, Eq, Hash)]
pub struct Account {
    pub accountID: String,
    pub balance: sqlx::types::BigDecimal,
    pub openDate: NaiveDate,
}

#[derive(PartialEq, sqlx::FromRow, Debug, Clone)]
pub struct SavingAccount {
    pub accountID: String,
    pub balance: sqlx::types::BigDecimal,
    pub openDate: NaiveDate,
    pub interest: f32,
    pub currencyType: String,
}

#[derive(PartialEq, sqlx::FromRow, Debug, Clone)]
pub struct CheckingAccount {
    pub accountID: String,
    pub balance: sqlx::types::BigDecimal,
    pub openDate: NaiveDate,
    pub overdraft: sqlx::types::BigDecimal,
}

#[derive(PartialEq, Eq, sqlx::FromRow, Debug, Clone, Hash)]
pub struct Loan {
    pub loanID: String,
    pub amount: sqlx::types::BigDecimal,
    pub subbranchName: String,
}

#[derive(PartialEq, Eq, sqlx::FromRow, Debug, Clone, Hash)]
pub struct Payment {
    pub loanID: String,
    pub date: NaiveDate,
    pub amount: sqlx::types::BigDecimal,
}

#[derive(PartialEq, Eq, sqlx::FromRow, Debug, Clone)]
pub struct ReceiveLoan {
    pub loanID: String,
    pub clientID: String,
}

#[derive(PartialEq, Eq, sqlx::FromRow, Debug, Clone)]
pub struct Subbranch {
    pub subbranchName: String,
    pub city: String,
    pub subbranchAsset: sqlx::types::BigDecimal,
}
