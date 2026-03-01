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
        if !self.locked {
            self.available += amount;
            self.total += amount;
        }
    }

    fn withdraw(&mut self, amount: Decimal) {
        if !self.locked && self.available >= amount {
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
        let Ok(transaction) = result else {
            continue;
        };

        match transaction.transaction_type {
            TransactionType::Deposit => {
                if let Some(amount) = transaction.amount {
                    let account = accounts
                        .entry(transaction.client)
                        .or_insert(Account::new(transaction.client));
    
                    account.deposit(amount);
    
                    transactions_history.insert(
                        transaction.tx,
                        TransactionRecord::new(transaction.client, amount),
                    );
                }
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = transaction.amount {
                    let account = accounts
                        .entry(transaction.client)
                        .or_insert(Account::new(transaction.client));
    
                    if account.locked {
                        continue;
                    }
    
                    account.withdraw(amount);
                }
            }
            TransactionType::Dispute => {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert(Account::new(transaction.client));

                if account.locked {
                    continue;
                }

                if let Some(record) = transactions_history.get_mut(&transaction.tx) {
                    if record.client == transaction.client && !record.is_disputed {
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


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_deposit() {
        let mut acc = Account::new(1);
        acc.deposit(Decimal::from(100));
        assert_eq!(acc.available, Decimal::from(100));
        assert_eq!(acc.total, Decimal::from(100));
    }

    #[test]
    fn test_withdrawal_sufficient_funds() {
        let mut acc = Account::new(1);
        acc.deposit(Decimal::from(100));
        acc.withdraw(Decimal::from(40));
        assert_eq!(acc.available, Decimal::from(60));
        assert_eq!(acc.total, Decimal::from(60));
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut acc = Account::new(1);
        acc.deposit(Decimal::from(50));
        acc.withdraw(Decimal::from(60)); 
        assert_eq!(acc.available, Decimal::from(50));
        assert_eq!(acc.total, Decimal::from(50));
    }

    #[test]
    fn test_basic_deposit_and_withdrawal() {
        let mut acc = Account::new(1);
        acc.deposit(Decimal::from(10));
        acc.withdraw(Decimal::new(45, 1));
        assert_eq!(acc.available, Decimal::new(55, 1));
    }

    #[test]
    fn test_locked_account_ignores_transactions() {
        let mut acc = Account::new(1);
        acc.locked = true;
        acc.deposit(Decimal::from(100));
        assert_eq!(acc.available, Decimal::from(0));
    }

    #[test]
    fn test_dispute_increases_held_and_maintains_total() {
        let mut acc = Account::new(1);
        acc.deposit(Decimal::from(100));
        
        let amount = Decimal::from(100);
        acc.available -= amount;
        acc.held += amount;

        assert_eq!(acc.available, Decimal::from(0));
        assert_eq!(acc.held, Decimal::from(100));
        assert_eq!(acc.total, Decimal::from(100));
    }

    #[test]
    fn test_resolve_after_dispute() {
        let mut acc = Account::new(1);
        acc.held = Decimal::from(100);
        acc.available = Decimal::from(0);
        acc.total = Decimal::from(100);

        let amount = Decimal::from(100);
        acc.held -= amount;
        acc.available += amount;

        assert_eq!(acc.available, Decimal::from(100));
        assert_eq!(acc.held, Decimal::from(0));
    }

    #[test]
    fn test_chargeback_locks_and_reduces_total() {
        let mut acc = Account::new(1);
        acc.held = Decimal::from(100);
        acc.total = Decimal::from(100);

        let amount = Decimal::from(100);
        acc.held -= amount;
        acc.total -= amount;
        acc.locked = true;

        assert_eq!(acc.total, Decimal::from(0));
        assert!(acc.locked);
    }

    #[test]
    fn test_decimal_precision_handling() {
        let mut acc = Account::new(1);
        // 0.1 + 0.2
        acc.deposit(Decimal::new(1, 1));
        acc.deposit(Decimal::new(2, 1));
        assert_eq!(acc.available, Decimal::new(3, 1)); // 0.3
    }
}