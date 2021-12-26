#![cfg(feature = "test-bpf")]

use std::assert;

use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::{instruction::Instruction, pubkey::Pubkey, sysvar};
use solana_program_test::{tokio, BanksClient};
use solana_sdk::signature::{Keypair, Signer};

// #[test]
// async test_mint() {
//     assert!(true);
// }
