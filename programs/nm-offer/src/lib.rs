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

        let offer_data = &mut ctx.accounts.offer_data;
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
        let offer_data = &mut ctx.accounts.offer_data;
        offer_data.accept_offer_item(offer_index);

        Ok(())
    }

    pub fn reject_offer (ctx : Context<DealOffer>, offer_index: u8) -> ProgramResult {
        let offer_data = &mut ctx.accounts.offer_data;
        offer_data.remove_offer_item(offer_index);

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
pub struct AddOffer<'info> {
    #[account(mut, signer)]
    offeror : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(init_if_needed, payer=offeror, space = 8 + OFFERDATA_SIZE)]
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
pub struct DealOffer<'info> {
    #[account(mut, signer)]
    offeror : AccountInfo<'info>, 

    pool : Account<'info, Pool>,

    #[account(mut)]
    offer_data : Account<'info, OfferData>,

}
