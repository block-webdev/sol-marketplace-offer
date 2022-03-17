use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::{ instruction::AuthorityType};


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");



#[program]
pub mod nm_offer {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, _bump : u8,) -> ProgramResult {
        msg!("initialize");

        let pool = &mut ctx.accounts.pool;
        pool.owner = *ctx.accounts.owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.bump = _bump;

        Ok(())
    }

    pub fn add_offer(
        ctx : Context<AddOffer>,
        offer_amount_sol: u64,
        offer_nft_price: u64,
        listed_price: u64,
        ) -> ProgramResult {
        msg!("Add offer");

        let pool = &ctx.accounts.pool;

        let offer_data = &mut ctx.accounts.offer_data;
        offer_data.offerer = *ctx.accounts.owner.key;
        offer_data.pool = pool.key();

        offer_data.listed_nft_mint = *ctx.accounts.listed_nft_mint.key;
        offer_data.listed_nft_account = *ctx.accounts.listed_nft_account.key;

        offer_data.offer_amount_sol = offer_amount_sol;
        offer_data.offer_nft_price = offer_nft_price;

        offer_data.offer_nft_mint = *ctx.accounts.offer_nft_mint.key;
        offer_data.offer_nft_account = *ctx.accounts.offer_nft_account.key;

        offer_data.listed_price = listed_price;

        Ok(())
    }

    pub fn update_offer(
        ctx : Context<UpdateOffer>,
        offer_amount_sol: u64,
        offer_nft_price: u64,
        ) -> ProgramResult {
        msg!("Update offer");

        let pool = &ctx.accounts.pool;
        let last_offer_data = &mut ctx.accounts.last_offer_data;
        if last_offer_data.pool != pool.key() {
            msg!("Not match owner");
            return Err(PoolError::InvalidPoolAccount.into());
        }

        let cur_offer_price = offer_amount_sol + offer_nft_price;
        let last_offer_price = last_offer_data.offer_amount_sol + last_offer_data.offer_nft_price;

        if cur_offer_price > last_offer_price {
            last_offer_data.offerer = *ctx.accounts.owner.key;
            last_offer_data.pool = pool.key();

            last_offer_data.listed_nft_mint = *ctx.accounts.listed_nft_mint.key;
            last_offer_data.listed_nft_account = *ctx.accounts.listed_nft_account.key;

            last_offer_data.offer_amount_sol = offer_amount_sol;
            last_offer_data.offer_nft_price = offer_nft_price;

            last_offer_data.offer_nft_mint = *ctx.accounts.offer_nft_mint.key;
            last_offer_data.offer_nft_account = *ctx.accounts.offer_nft_account.key;
        }

        Ok(())
    }

}

#[account]
#[derive(Default)]
pub struct Pool {
    pub owner : Pubkey,
    pub rand : Pubkey,
    pub bump : u8,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct Initialize<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, payer = owner, seeds = [(*rand.key).as_ref()], bump = _bump)]
    pool : Account<'info, Pool>,

    rand : AccountInfo<'info>,

    system_program : Program<'info,System>,
}

pub const OFFERDATA_SIZE : usize = 32 + 32 + 32 + 32 + 8 + 8 + 32 + 32 + 8;

#[account]
pub struct OfferData {
    pub offerer: Pubkey,
    pub pool : Pubkey,
    pub listed_nft_mint: Pubkey,
    pub listed_nft_account: Pubkey,

    pub offer_amount_sol: u64,
    pub offer_nft_price: u64, // sol unit

    pub offer_nft_mint: Pubkey,
    pub offer_nft_account: Pubkey,

    pub listed_price: u64, // sol unit
}


#[derive(Accounts)]
pub struct AddOffer<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(init, payer=owner, space = 8 + OFFERDATA_SIZE)]
    offer_data : Account<'info, OfferData>,

    // offer infos start------------
    #[account(mut,owner=spl_token::id())]
    listed_nft_mint : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    listed_nft_account : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    offer_nft_mint : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    offer_nft_account : AccountInfo<'info>, 
    // end ------------

    system_program : Program<'info,System>,
}


#[derive(Accounts)]
pub struct UpdateOffer<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    pool : Account<'info,Pool>,

    #[account(mut)]
    last_offer_data : Account<'info, OfferData>,

    // offer infos start------------
    #[account(mut,owner=spl_token::id())]
    listed_nft_mint : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    listed_nft_account : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    offer_nft_mint : AccountInfo<'info>, 

    #[account(mut,owner=spl_token::id())]
    offer_nft_account : AccountInfo<'info>, 
    // end ------------

    system_program : Program<'info,System>,
}


#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid metadata")]
    InvalidMetadata,

    #[msg("Invalid pool account")]
    InvalidPoolAccount,

    #[msg("Invalid time")]
    InvalidTime,

    #[msg("Invalid Period")]
    InvalidPeriod,

    #[msg("Already unstaked")]
    AlreadyUnstaked,
}