use csv::{ReaderBuilder, Trim};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use std;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Debug, Serialize)]
struct Account {
    client: u16,

    #[serde(serialize_with = "format_decimal")]
    available: Decimal,

    #[serde(serialize_with = "format_decimal")]
    held: Decimal,

    #[serde(serialize_with = "format_decimal")]
    total: Decimal,

    locked: bool,
}

struct TransactionRecord {
    client: u16,
    amount: Decimal,
    is_disputed: bool,
}

impl TransactionRecord {
    fn new(client: u16, amount: Decimal) -> Self {
        Self {
            client,
            amount,
            is_disputed: false,
        }
    }
}

impl Account {
    fn new(client: u16) -> Self {
        Self {
            client,
            available: Decimal::from(0),
            held: Decimal::from(0),
            total: Decimal::from(0),
            locked: false,
        }
    }

    fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total += amount;
    }

    fn withdraw(&mut self, amount: Decimal) {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
        }
    }
}

fn format_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{:.4}", value);
    serializer.serialize_str(&s)
}

fn main() {
    let file_path = std::env::args().nth(1).expect("No CSV file path provided");

    let mut accounts: HashMap<u16, Account> = HashMap::new();
    let mut transactions_history: HashMap<u32, TransactionRecord> = HashMap::new();

    let mut transactions_reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(&file_path)
        .unwrap();

    for result in transactions_reader.deserialize() {
        let transaction: Transaction = result.unwrap();

        match transaction.transaction_type {
            TransactionType::Deposit => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                account.deposit(transaction.amount.unwrap());

                transactions_history.insert(
                    transaction.tx,
                    TransactionRecord::new(transaction.client, transaction.amount.unwrap()),
                );
            }
            TransactionType::Withdrawal => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                account.withdraw(transaction.amount.unwrap());
            }
            TransactionType::Dispute => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                if let Some(record) = transactions_history.get_mut(&transaction.tx) {
                    if record.client == transaction.client {
                        account.held += record.amount;
                        account.available -= record.amount;
                        record.is_disputed = true;
                    }
                }
            }
            TransactionType::Resolve => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                if let Some(record) = transactions_history.get_mut(&transaction.tx) {
                    if record.is_disputed && record.client == transaction.client {
                        account.held -= record.amount;
                        account.available += record.amount;
                        record.is_disputed = false;
                    }
                }
            }
            TransactionType::Chargeback => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                if let Some(record) = transactions_history.get_mut(&transaction.tx) {
                    if record.is_disputed && record.client == transaction.client {
                        account.held -= record.amount;
                        account.total -= record.amount;
                        record.is_disputed = false;
                        account.locked = true;
                    }
                }
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for account in accounts.values() {
        wtr.serialize(account).unwrap();
    }
    wtr.flush().unwrap();
}
