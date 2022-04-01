use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer, Token};
use spl_token::{ instruction::AuthorityType};
use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};

pub mod account;
pub mod constant;
pub mod error;

use account::*;
use constant::*;
use error::*;


declare_id!("8SiHwRFc5nJ9QjvWvDGvtQqteBCNTT1s1DMw7mAzE3Cv");



#[program]
pub mod nm_offer {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, _bump : u8,) -> ProgramResult {
        msg!("initialize");

        let pool = &mut ctx.accounts.pool;
        pool.owner = *ctx.accounts.owner.key;
        pool.bump = _bump;

        Ok(())
    }

    pub fn create_offerdata(ctx: Context<CreateOfferData>) -> ProgramResult {
        let offer_data = &mut ctx.accounts.offer_data.load_init()?;

        offer_data.pool = *ctx.accounts.pool.to_account_info().key;

        Ok(())
    }

    pub fn add_offer(
        ctx : Context<AddOffer>,
        collection_id: u32,
        nft_id: u32,
        offer_amount_sol: u64,
        offer_nft_price: u64,
        listed_price: u64,
        floor_price: u64,
        offer_nft_mint: [Pubkey; 5],    // BUY_MAX_NFT_COUNT
        offer_nft_account: [Pubkey; 5], // BUY_MAX_NFT_COUNT
        offer_nft_count: u8,
        ) -> ProgramResult {
        msg!("Add offer data");

        let offer_data = &mut ctx.accounts.offer_data.load_mut()?;

        if offer_data.offer_item_count < MAX_OFFER_COUNT as u8 {
            if offer_nft_count != 0 || offer_amount_sol >= floor_price {
                let pool = &ctx.accounts.pool;

                offer_data.collection_id = collection_id;
                offer_data.nft_id = nft_id;

                offer_data.offeror = *ctx.accounts.offeror.key;
                offer_data.pool = pool.key();

                let offer_item = OfferItem {
                    offer_amount_sol: offer_amount_sol,
                    offer_nft_price: offer_nft_price,
                    offer_nft_mint: offer_nft_mint,
                    offer_nft_account: offer_nft_account,
                    offer_nft_count: offer_nft_count,
                };
                offer_data.add_offer_item(offer_item);

                offer_data.listed_price = listed_price;
                offer_data.floor_price = floor_price;
            }
        }

        Ok(())
    }

    pub fn accept_offer (ctx : Context<DealOffer>, offer_index: u8) -> ProgramResult {
        let offer_data = &mut ctx.accounts.offer_data.load_mut()?;
        offer_data.accept_offer_item(offer_index);

        Ok(())
    }

    pub fn reject_offer (ctx : Context<DealOffer>, offer_index: u8) -> ProgramResult {
        let offer_data = &mut ctx.accounts.offer_data.load_mut()?;
        offer_data.remove_offer_item(offer_index);

        Ok(())
    }

    pub fn cancel_offer (ctx : Context<DealOffer>, offer_index: u8) -> ProgramResult {
        let offer_data = &mut ctx.accounts.offer_data.load_mut()?;
        require!(offer_data.offeror == *ctx.accounts.offeror.key, NMError::InvalidOwner);

        offer_data.remove_offer_item(offer_index);

        Ok(())
    }

    
    pub fn add_nft(
        ctx : Context<AddNft>,
        nft_addr: Pubkey,
        owner: Pubkey,
        collection_id: u32,
        nft_id: u32,
        pool: Pubkey,
        ) -> ProgramResult {
        msg!("Add nft data");

        let nft_data = &mut ctx.accounts.nft_data;

        nft_data.nft_addr = nft_addr;
        nft_data.owner = owner;
        nft_data.collection_id = collection_id;
        nft_data.nft_id = nft_id;
        nft_data.pool = pool;

        Ok(())
    }

    pub fn remove_nft(
        ctx : Context<RemoveNft>,
        ) -> ProgramResult {
        msg!("Remove nft data");

        Ok(())
    }

    #[access_control(user(&ctx.accounts.nft_data, &ctx.accounts.source_account))]
    pub fn buy_nft(
        ctx: Context<BuyNftStep>,
        global_bump: u8,
        offer_index: u8,
    ) -> ProgramResult {

        require!(ctx.accounts.nft_on_sale.owner == *ctx.accounts.pool.to_account_info().key, NMError::NotPuttedOnSale);

        let offer_data = &mut ctx.accounts.offer_data.load_mut()?;
        require!(offer_index < offer_data.offer_item_count, NMError::OverflowOfferCount);

        let pool = &mut ctx.accounts.pool;
        let offer_item = offer_data.offer_items[offer_index as usize];

        let account_length = ctx.remaining_accounts.len();
        require!(account_length == offer_item.offer_nft_count as usize * 2, NMError::OverflowTokenAccountCount);

        // check buying state
        let buying_state = &mut ctx.accounts.buying_state;
        if buying_state.paid_sol == false {
            // if not paid in sol, pay in sol.
            invoke(
                &transfer(
                    ctx.accounts.buyer.key,
                    pool.to_account_info().key,
                    offer_item.offer_amount_sol
                ),
                &[ctx.accounts.buyer.clone(),
                pool.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone()]
            )?;
            buying_state.paid_sol = true;
        }

        let source_account = &mut ctx.accounts.source_account;
        let dest_account = &mut ctx.accounts.dest_account;

        // check if source_account is paid
        let found_idx_elem1 = offer_item.offer_nft_account.
                                    iter().
                                    position(|&x| x == *source_account.key ).
                                    unwrap_or(offer_item.offer_nft_count as usize);

        require!(found_idx_elem1 < offer_item.offer_nft_count as usize, NMError::InvalidSourceAccount);

        let found_idx_elem2 = buying_state.paid_nft_account_list.
                                    iter().
                                    position(|&x| x == *source_account.key ).
                                    unwrap_or(offer_item.offer_nft_count as usize);
        if found_idx_elem2 == offer_item.offer_nft_count as usize {
            // not found, not paid
            let cpi_accounts = Transfer {
                from: source_account.to_account_info(),
                to: dest_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            };
            let transfer_ctx1 = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
            token::transfer(
                transfer_ctx1,
                1
            )?;

            buying_state.add_paid_nft(*source_account.key);

            if buying_state.paid_nft_count == offer_item.offer_nft_count {
                // paid finally
                // withdraw to dest account

                let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
                let signer = &[&seeds[..]];
                let cpi_accounts1 = Transfer {
                    from: dest_account.to_account_info(),
                    to: source_account.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info()
                };
                let transfer_ctx2 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts1, signer);
                token::transfer(
                    transfer_ctx2,
                    1
                )?;
            }
        }

        Ok(())
    }

    pub fn put_token_on_sale(ctx: Context<PutOnSale>,
        collection_id: u32,
        nft_id: u32,
        price: u64) -> ProgramResult {

        let nft_on_sale = &mut ctx.accounts.nft_on_sale;
        nft_on_sale.owner = *ctx.accounts.owner.key;
        nft_on_sale.collection_id = collection_id;
        nft_on_sale.nft_id = nft_id;
        nft_on_sale.price = price;

        let cpi_accounts = Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: ctx.accounts.dest_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let transfer_ctx1 = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(
            transfer_ctx1,
            1
        )?;

        Ok(())
    }

    pub fn cancel_token_from_sale(ctx: Context<CancelFromSale>,global_bump: u8) -> ProgramResult {

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts1 = Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: ctx.accounts.dest_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info()
        };
        let transfer_ctx2 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts1, signer);
        token::transfer(
            transfer_ctx2,
            1
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct Initialize<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, payer = owner, seeds = [GLOBAL_AUTHORITY_SEED.as_ref()], bump = _bump)]
    pool : Account<'info, Pool>,

    system_program : Program<'info,System>,
}

#[derive(Accounts)]
pub struct CreateOfferData<'info> {
    #[account(mut, signer)]
    initializer : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(zero)]
    offer_data : AccountLoader<'info, OfferData>,
}

#[derive(Accounts)]
pub struct AddOffer<'info> {
    #[account(mut, signer)]
    offeror : AccountInfo<'info>, 

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


#[derive(Accounts)]
pub struct DealOffer<'info> {
    #[account(mut, signer)]
    offeror : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(mut)]
    offer_data : AccountLoader<'info, OfferData>,

}

#[derive(Accounts)]
pub struct AddNft<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(init_if_needed, payer=owner)]
    nft_data : Account<'info, NftData>,

    system_program : Program<'info,System>,
}


#[derive(Accounts)]
pub struct RemoveNft<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    #[account(mut)]
    pool : Account<'info, Pool>,

    #[account(mut, close = receiver)]
    nft_data : Account<'info, NftData>,

    #[account(mut)]
    pub receiver: SystemAccount<'info>
}


#[derive(Accounts)]
#[instruction(_bump: u8)]
pub struct BuyNftStep<'info> {
    #[account(mut, signer)]
    buyer : AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = _bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    buying_state : Account<'info, BuyingState>,

    #[account(mut)]
    offer_data : AccountLoader<'info, OfferData>,

    source_account : AccountInfo<'info>,
    dest_account : AccountInfo<'info>,

    nft_on_sale: Account<'info, NftOnSale>,

    #[account(mut, close = receiver)]
    nft_data : Account<'info, NftData>,

    #[account(mut)]
    pub receiver: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,

    system_program : Program<'info,System>,
}

#[derive(Accounts)]
pub struct PutOnSale<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, payer = owner)]
    nft_on_sale: Account<'info, NftOnSale>,

    source_account : AccountInfo<'info>,
    dest_account : AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    system_program : Program<'info,System>,
}


#[derive(Accounts)]
pub struct CancelFromSale<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : Account<'info, Pool>,

    #[account(mut, close = receiver)]
    nft_on_sale: Account<'info, NftOnSale>,

    #[account(mut)]
    pub receiver: SystemAccount<'info>,

    source_account : AccountInfo<'info>,
    dest_account : AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    system_program : Program<'info,System>,
}

// Access control modifiers
fn user(nft_data: &Account<NftData>, user: &AccountInfo) -> Result<()> {
    require!(nft_data.owner == *user.key, NMError::InvalidOwner);
    Ok(())
}