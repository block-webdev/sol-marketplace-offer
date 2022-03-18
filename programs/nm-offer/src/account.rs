use anchor_lang::prelude::*;


use crate::constants::*;

pub const OFFERITEM_SIZE : usize = 8 + 8 + 32 + 32;

#[zero_copy]
#[derive(Default)]
pub struct OfferItem {
    pub offer_amount_sol: u64,
    pub offer_nft_price: u64, // sol unit

    pub offer_nft_mint: Pubkey,
    pub offer_nft_account: Pubkey,
}

pub const OFFERDATA_SIZE : usize = 32 + 32 + 32 + ((8 + OFFERITEM_SIZE) * MAX_OFFER_COUNT) + 8 + 32 + 8;

#[account(zero_copy)]
pub struct OfferData {
    // listed nft info for selling
    pub listed_nft_mint: Pubkey,
    pub listed_nft_account: Pubkey,

    // public key of person sending offer.
    pub offeror: Pubkey,

    // offer items
    pub offer_items: [OfferItem; MAX_OFFER_COUNT],
    pub offer_item_count: u64,                         // 8

    // pool info
    pub pool : Pubkey,

    // selling price
    pub listed_price: u64, // sol unit
}

impl OfferData {
    pub fn add_offer_item(&mut self, item: OfferItem) {
        self.offer_items[self.offer_item_count as usize] = item;
        self.offer_item_count += 1;
    }
}
