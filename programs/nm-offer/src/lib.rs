use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::{ instruction::AuthorityType};

pub mod account;
pub mod constants;

use account::*;
use constants::*;


declare_id!("8SiHwRFc5nJ9QjvWvDGvtQqteBCNTT1s1DMw7mAzE3Cv");



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

    pub fn init_offer_data(
        ctx : Context<InitOfferData>,
        offer_amount_sol: u64,
        offer_nft_price: u64,
        listed_price: u64,
        ) -> ProgramResult {
        msg!("Init offer data");

        let pool = &ctx.accounts.pool;

        let mut offer_data = ctx.accounts.offer_data.load_mut()?;
        offer_data.offeror = *ctx.accounts.offeror.key;
        offer_data.pool = pool.key();

        offer_data.listed_nft_mint = *ctx.accounts.listed_nft_mint.key;
        offer_data.listed_nft_account = *ctx.accounts.listed_nft_account.key;

        let offer_item = OfferItem {
            offer_amount_sol: offer_amount_sol,
            offer_nft_price: offer_nft_price,
            offer_nft_mint: *ctx.accounts.offer_nft_mint.key,
            offer_nft_account: *ctx.accounts.offer_nft_account.key,
        };
        offer_data.add_offer_item(offer_item);

        offer_data.listed_price = listed_price;

        Ok(())
    }

    pub fn add_offer(
        ctx : Context<AddOffer>,
        offer_amount_sol: u64,
        offer_nft_price: u64,
        ) -> ProgramResult {
        msg!("Add offer");

        let pool = &ctx.accounts.pool;
        let mut offer_data = ctx.accounts.offer_data.load_mut()?;
        if offer_data.pool != pool.key() {
            msg!("Not match owner");
            return Err(PoolError::InvalidPoolAccount.into());
        }

        if offer_data.offer_item_count < MAX_OFFER_COUNT as u64 {
            let offer_item = OfferItem {
                offer_amount_sol: offer_amount_sol,
                offer_nft_price: offer_nft_price,
                offer_nft_mint: *ctx.accounts.offer_nft_mint.key,
                offer_nft_account: *ctx.accounts.offer_nft_account.key,
            };
            offer_data.add_offer_item(offer_item);
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

#[derive(Accounts)]
pub struct InitOfferData<'info> {
    #[account(mut, signer)]
    offeror : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(init, payer=offeror, space = 8 + OFFERDATA_SIZE)]
    offer_data : AccountLoader<'info, OfferData>,

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
pub struct AddOffer<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(mut)]
    offer_data : AccountLoader<'info, OfferData>,

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