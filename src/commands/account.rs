// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Result;
use clap::{Parser, Subcommand};
use iota_wallet::{
    account::{
        types::{AccountAddress, Transaction},
        AccountHandle,
    },
    iota_client::{bee_message::output::TokenId, request_funds_from_faucet},
    AddressAndAmount, AddressNativeTokens, U256,
};

use std::str::FromStr;

#[derive(Parser)]
#[clap(version, long_about = None)]
#[clap(propagate_version = true)]
pub struct AccountCli {
    #[clap(subcommand)]
    pub command: AccountCommands,
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Generate a new address.
    Address,
    /// Print the account balance.
    Balance,
    /// Request funds from the faucet to the latest address, `url` is optional, default is `http://localhost:14265/api/plugins/faucet/v1/enqueue`
    Faucet { url: Option<String> },
    /// List the account addresses.
    ListAddresses,
    /// List the account transactions.
    ListTransactions,
    /// Send an amount to a bech32 address: `send atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r
    /// 1000000`
    Send { address: String, amount: u64 },
    /// Send a native token to a bech32 address: `send-native
    /// atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r
    /// 08e3a2f76cc934bc0cc21575b4610c1d7d4eb589ae0100000000000000000000000000000000 10`
    SendNative {
        address: String,
        token_id: String,
        native_token_amount: String,
    },
    /// Sync the account with the Tangle.
    Sync,
    /// Exit from the account prompt.
    Exit,
}

/// `list-transactions` command
pub async fn list_transactions_command(account_handle: &AccountHandle) -> Result<()> {
    let transactions = account_handle.list_transactions().await?;
    if transactions.is_empty() {
        println!("No transactions found");
    } else {
        transactions.iter().for_each(print_transaction);
    }
    Ok(())
}

/// `list-addresses` command
pub async fn list_addresses_command(account_handle: &AccountHandle) -> Result<()> {
    let addresses = account_handle.list_addresses().await.unwrap();
    if addresses.is_empty() {
        println!("No addresses found");
    } else {
        for address in addresses {
            print_address(account_handle, &address).await;
        }
    }
    Ok(())
}

// `sync` command
pub async fn sync_account_command(account_handle: &AccountHandle) -> Result<()> {
    let sync = account_handle.sync(None).await?;
    println!("Synced: {:?}", sync);
    Ok(())
}

// `address` command
pub async fn generate_address_command(account_handle: &AccountHandle) -> Result<()> {
    let address = account_handle.generate_addresses(1, None).await?;
    print_address(account_handle, &address[0]).await;
    Ok(())
}

// `balance` command
pub async fn balance_command(account_handle: &AccountHandle) -> Result<()> {
    println!("{:?}", account_handle.balance().await?);
    Ok(())
}

// `send` command
pub async fn send_command(account_handle: &AccountHandle, address: String, amount: u64) -> Result<()> {
    let outputs = vec![AddressAndAmount { address, amount }];
    let transfer_result = account_handle.send_amount(outputs, None).await?;
    println!("Transaction created: {:?}", transfer_result);
    Ok(())
}

// `send-native` command
pub async fn send_native_command(
    account_handle: &AccountHandle,
    address: String,
    token_id: String,
    native_token_amount: String,
) -> Result<()> {
    let outputs = vec![AddressNativeTokens {
        address,
        native_tokens: vec![(TokenId::from_str(&token_id)?, U256::from_dec_str(&native_token_amount)?)],
        ..Default::default()
    }];
    let transfer_result = account_handle.send_native_tokens(outputs, None).await?;
    println!("Transaction created: {:?}", transfer_result);
    Ok(())
}

// `faucet` command
pub async fn faucet_command(account_handle: &AccountHandle, url: Option<String>) -> Result<()> {
    let address = match account_handle.list_addresses().await?.last() {
        Some(address) => address.clone(),
        None => return Err(anyhow::anyhow!("Generate an address first!")),
    };
    let faucet_url = match &url {
        Some(faucet_url) => faucet_url,
        None => "http://localhost:14265/api/plugins/faucet/v1/enqueue",
    };
    println!(
        "{}",
        request_funds_from_faucet(faucet_url, &address.address().to_bech32()).await?
    );
    Ok(())
}

// `set-alias` command
// pub async fn set_alias_command(account_handle: &AccountHandle) -> Result<()> {
//     if let Some(matches) = matches.subcommand_matches("set-alias") {
//         let alias = matches.value_of("alias").unwrap();
//         account_handle.set_alias(alias).await?;
//     }
//     Ok(())
// }

fn print_transaction(transaction: &Transaction) {
    println!("TRANSACTION {:?}", transaction);
    // if let Some(MessagePayload::Transaction(tx)) = message.payload() {
    //     let TransactionEssence::Regular(essence) = tx.essence();
    //     println!("--- Value: {:?}", essence.value());
    // }
    // println!("--- Timestamp: {:?}", message.timestamp());
    // println!(
    //     "--- Broadcasted: {}, confirmed: {}",
    //     message.broadcasted(),
    //     match message.confirmed() {
    //         Some(c) => c.to_string(),
    //         None => "unknown".to_string(),
    //     }
    // );
}

pub async fn print_address(_account_handle: &AccountHandle, address: &AccountAddress) {
    println!("ADDRESS {:?}", address.address().to_bech32());
    // println!("Address balance: {}", address.balance());
    println!("--- Index: {}", address.key_index());
    println!("--- Change address: {}", address.internal());
    // println!("--- Address outputs: {}", address.output_ids());
}