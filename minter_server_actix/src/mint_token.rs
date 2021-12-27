use std::str::FromStr;

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anyhow::Result;
use clap::ArgMatches;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_program::{program_pack::Pack, system_instruction, sysvar};
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

use metaplex_token_metadata::{
    instruction::{
        create_master_edition, create_metadata_accounts,
        mint_new_edition_from_master_edition_via_token, puff_metadata_account,
        update_metadata_accounts,
    },
    state::{
        get_reservation_list, Data, Edition, Key, MasterEditionV1, MasterEditionV2, Metadata,
        EDITION, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH, PREFIX,
    },
};

pub fn init_and_mint_to_dest(
    minter: &Keypair,
    dest_pubkey: &Pubkey,
    matches: ArgMatches,
    rpc_client: RpcClient,
) -> Result<Pubkey> {
    let uri = matches.value_of("uri").unwrap();
    println!("Value for URI: {}", &uri);

    let name = matches.value_of("name").unwrap();
    println!("Value for name: {}", &name);

    let symbol = matches.value_of("symbol").unwrap();
    println!("Value for symbol: {}", &symbol);

    let token_mint_account_rent =
        rpc_client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN as usize)?;

    let nft = Keypair::new();
    let token_program_id = spl_token::id();
    let minter_pubkey = minter.pubkey();
    let ntf_pubkey = nft.pubkey();
    let ata_nft =
        spl_associated_token_account::get_associated_token_address(&dest_pubkey, &ntf_pubkey);

    let program_key = metaplex_token_metadata::id();

    let metadata_seeds = &[
        PREFIX.as_bytes(),
        &program_key.as_ref(),
        ntf_pubkey.as_ref(),
    ];
    let (metadata_key, _) = Pubkey::find_program_address(metadata_seeds, &program_key);

    println!("NFT TOKEN: {} for {} ", ntf_pubkey, dest_pubkey);
    println!("metadata_key {}", metadata_key);
    let (hash, _) = rpc_client.get_recent_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &minter.pubkey(),
                &ntf_pubkey,
                token_mint_account_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &token_program_id,
                &ntf_pubkey,
                &minter_pubkey,
                None,
                0,
            )
            .unwrap(),
            metaplex_token_metadata::instruction::create_metadata_accounts(
                program_key,
                metadata_key,
                ntf_pubkey,
                minter_pubkey,
                minter.pubkey(),
                minter_pubkey,
                name.to_string(),
                symbol.to_string(),
                uri.to_string(),
                None,
                0,
                true,
                true,
            ),
        ],
        Some(&minter.pubkey()),
        &[minter, &nft],
        hash,
    );

    rpc_client
        .send_and_confirm_transaction_with_spinner_and_config(
            &transaction,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )
        .unwrap_or_else(|e| panic!("EEEEEEEEE {:?}", e));

    println!("DONE SETUP NFT");

    let (hash, _) = rpc_client.get_recent_blockhash()?;
    let transfer_tx = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::create_associated_token_account(
                &minter.pubkey(),
                &dest_pubkey,
                &ntf_pubkey,
            ),
            spl_token::instruction::mint_to(
                &token_program_id,
                &ntf_pubkey,
                &ata_nft,
                &minter_pubkey,
                &[&minter_pubkey],
                1,
            )
            .unwrap(),
            spl_token::instruction::set_authority(
                &token_program_id,
                &ntf_pubkey,
                None,
                spl_token::instruction::AuthorityType::MintTokens,
                &minter_pubkey,
                &[&minter_pubkey],
            )
            .unwrap(),
        ],
        Some(&minter.pubkey()),
        &[minter],
        hash,
    );

    rpc_client
        .send_and_confirm_transaction_with_spinner_and_config(
            &transfer_tx,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )
        .unwrap_or_else(|e| panic!("EEEEEEEEE {:?}", e));

    // create_metadata_account(ntf_pubkey, minter, &rpc_client).await?;

    let token_account = rpc_client.get_token_account(&ata_nft).unwrap().unwrap();
    println!("{:?} ", token_account);

    Ok(ntf_pubkey)
}
pub async fn create_metadata_account(
    ntf_pubkey: Pubkey,
    minter: &Keypair,
    rpc_client: &RpcClient,
) -> Result<()> {
    let program_key = metaplex_token_metadata::id();

    let metadata_seeds = &[
        PREFIX.as_bytes(),
        &program_key.as_ref(),
        ntf_pubkey.as_ref(),
    ];
    let (metadata_key, _) = Pubkey::find_program_address(metadata_seeds, &program_key);

    let minter_pubkey = minter.pubkey();
    let name = "BPH NFT TOKEN";
    let symbol = "BPHTK";
    let uri = "https://bafkreihxmkyhpdfy4vuiyc5efsbk3d4ij5bwawpohlfai3ie7pvarwbaam.ipfs.dweb.link/";
    let creators = None;

    println!("metadata_key {}", metadata_key);
    let (hash, _) = rpc_client.get_recent_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[
            metaplex_token_metadata::instruction::create_metadata_accounts(
                program_key,
                metadata_key,
                ntf_pubkey,
                minter_pubkey,
                minter_pubkey,
                minter_pubkey,
                name.to_string(),
                symbol.to_string(),
                uri.to_string(),
                creators,
                0,
                true,
                true,
            ),
        ],
        Some(&minter_pubkey),
        &[minter],
        hash,
    );

    rpc_client
        .send_and_confirm_transaction_with_spinner_and_config(
            &transaction,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )
        .unwrap_or_else(|e| panic!("EEEEEEEEE {:?}", e));

    Ok(())
}
