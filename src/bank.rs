use std::collections::HashMap;
use serde::Deserialize;

#[derive(PartialEq, Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
	Withdrawal,
	Deposit,
	Dispute,
	Resolve,
	Chargeback,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
	pub tx_type: TransactionType,
	pub client: u16,
	pub tx: u32,
    // this will allow deposits and withdrawals to have an empty amount field as well, but there is no harm in them, as it assumes a value of 0.0 and ignores them
    #[serde(deserialize_with = "default_if_empty")]
	pub amount: f32,
}

fn default_if_empty<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    Option::<T>::deserialize(de).map(|x| x.unwrap_or_else(|| T::default()))
}

#[derive(Debug)]
pub struct Bank {
	bank: HashMap<u16, Client>,
}

impl Bank {
	pub fn new() -> Bank {
    	Bank { bank: HashMap::new(), }
	}   

	pub fn add_client(&mut self, client_id: u16) {
    	if !self.bank.contains_key(&client_id) {
        	let client = Client::new(client_id);
        	self.bank.insert(client_id, client);
    	}   
	}   

    // Insert a transaction into the bank
    // This assumes txn ID + client ID is the unique primary key for a txn
	pub fn insert_txn(&mut self, txn: Transaction) {
        if !self.bank.contains_key(&txn.client) {
            // I'm assuming that the first transaction must be a deposit to open a new account
            if txn.tx_type == TransactionType::Deposit {
                self.add_client(txn.client);
            }
        }
    	if self.bank.contains_key(&txn.client) {
        	self.bank.get_mut(&txn.client).unwrap().process_txn(txn);
    	}
	}

    pub fn to_string(&self) -> String {
        let mut client_string: String = "".to_owned();
        for client_id in self.bank.keys() {
            let clientstr = self.bank.get(&client_id).unwrap().to_string();
            client_string.push_str(&clientstr);
            client_string.push_str("\n");
        }

        format!("client, available, held, total, locked\n{}", client_string)
    }
}

#[derive(Debug)]
pub struct Client {
	client: u16,
	txns: HashMap<u32, Transaction>,
	available: f32,
	held: f32,
	locked: bool,
    disputes: HashMap<u32, Transaction>
}

impl Client {
	pub fn new(client: u16) -> Client {
    	Client {
        	client: client,
        	txns: HashMap::new(),
        	available: 0.0,
        	held: 0.0,
        	locked: false,
            disputes: HashMap::new()
    	}
	}

	pub fn to_string(&self) -> String {
    	format!("{}, {:.4}, {:.4}, {:.4}, {}", self.client, self.available, self.held, self.available + self.held, self.locked)
	}

	pub fn process_txn(&mut self, txn: Transaction) {
        // if the account is locked, no txns can be processed. There is currently no way to unlock a locked account
        if !self.locked {
        	match txn.tx_type {
            	TransactionType::Withdrawal => self.withdrawal(txn),
            	TransactionType::Deposit => self.deposit(txn),
        	    TransactionType::Dispute => self.dispute(txn.tx),
            	TransactionType::Resolve => self.resolve(txn.tx),
            	TransactionType::Chargeback => self.chargeback(txn.tx)
    	    }
        }
	}

    // Note there is intentionally no protection on the account going negative. I'm assuming this is allowed.
    // Alternatively, a withdrawal could fail if it would make the available amount go negative.
	pub fn withdrawal(&mut self, txn: Transaction) {
        // I'm assuming every withdrawal must have a tx ID that is unique from all other client's tx IDs
        // If not, discard the txn as duplicate / mistake
        // Also ignore withdrawals with an amount of 0.0 as they are not useful
        if !self.txns.contains_key(&txn.tx) && txn.amount != 0.0 {
        	self.available -= txn.amount;
            self.txns.insert(txn.tx, txn);
        }
	}

	fn deposit(&mut self, txn: Transaction) {
        // I'm assuming every deposit must have a tx ID that is unique from all other client's tx IDs
        // If not, discard the txn as duplicate / mistake
        // Also ignore deposits with an amount of 0.0 as they are not useful
        if !self.txns.contains_key(&txn.tx) && txn.amount != 0.0 {
    	    self.available += txn.amount;
            self.txns.insert(txn.tx, txn);
        }
	}

	fn dispute(&mut self, tx: u32) {
        // if the tx is not found for this client, ignore
        if self.txns.contains_key(&tx) {
            let txn = self.txns.get(&tx).unwrap();
            // Given the description of the problem, I am assuming only deposits can be disputed
            if txn.tx_type == TransactionType::Deposit {
                let amount = txn.amount;
                self.available -= amount;
                self.held += amount;
                self.disputes.insert(tx, *txn);
            }
        }
	}

	fn resolve(&mut self, tx: u32) {
        // if there is no active dispute for this client & tx id, ignore
        if self.disputes.contains_key(&tx) {
            let txn = self.disputes.remove(&tx).unwrap();
            self.available += txn.amount;
            self.held -= txn.amount;
        }
	}

	fn chargeback(&mut self, tx: u32) {
        // if there is no active dispute for this client & tx id, ignore
        if self.disputes.contains_key(&tx) {
            let txn = self.disputes.remove(&tx).unwrap();
            self.held -= txn.amount;
            self.locked = true;
        }
	}
}

