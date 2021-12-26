use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod minter_machine {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        let program_id = Pubkey::new_unique();
        let metadata_account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let name = "name".to_string();
        let symbol = "Symbol".to_string();
        let uri = "uri".to_string();
        let creators = None;
        let seller_fee_basis_points = 1;
        let update_authority_is_signer = false;
        let is_mutable = false;
        let create = metaplex_token_metadata::instruction::create_metadata_accounts(
            program_id,
            metadata_account,
            mint,
            mint_authority,
            payer,
            update_authority,
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            update_authority_is_signer,
            is_mutable,
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
