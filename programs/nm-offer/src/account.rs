use anchor_lang::prelude::*;


use crate::constant::*;



#[account]
#[derive(Default)]
pub struct Pool {
    pub owner : Pubkey,
    pub bump : u8,
}

pub const OFFERITEM_SIZE : usize = 8 + 8 + (32 * BUY_MAX_NFT_COUNT) * 2 + 1;

#[zero_copy]
#[derive(Default, AnchorSerialize, AnchorDeserialize)]
pub struct OfferItem {
    pub offer_amount_sol: u64,
    pub offer_nft_price: u64, // sol unit

    pub offer_nft_mint: [Pubkey; BUY_MAX_NFT_COUNT],
    pub offer_nft_account: [Pubkey; BUY_MAX_NFT_COUNT],
    pub offer_nft_count: u8, // must be less than BUY_MAX_NFT_COUNT
}

pub const OFFERDATA_SIZE : usize = 4 + 4 + 32 + (OFFERITEM_SIZE * MAX_OFFER_COUNT) + 1 + 32 + 8 + 8;

#[account(zero_copy)]
#[repr(packed)]
pub struct OfferData {
    // listed nft info for selling
    pub collection_id : u32,    // collection id containing nft
    pub nft_id : u32,   // nft id for selling

    // public key of person sending offer.
    pub offeror: Pubkey,    // person sent offer

    // offer items
    pub offer_items: [OfferItem; MAX_OFFER_COUNT],
    pub offer_item_count: u8,

    // pool info
    pub pool : Pubkey,

    // selling price
    pub listed_price: u64, // sol unit
    pub floor_price: u64, // sol unit
}

impl Default for OfferData {
    #[inline]
    fn default() -> OfferData {
        OfferData {
            collection_id : 0,    // collection id containing nft
            nft_id : 0,   // nft id for selling

            // public key of person sending offer.
            offeror: Pubkey::default(),    // person sent offer

            // offer items
            offer_items: [
                OfferItem {
                    ..Default::default()
                }; MAX_OFFER_COUNT
            ],
            offer_item_count: 0,

            // pool info
            pool : Pubkey::default(),

            // selling price
            listed_price: 0, // sol unit
            floor_price: 0, // sol unit
        }
    }
}

impl OfferData {
    pub fn add_offer_item(&mut self, item: OfferItem) {
        self.offer_items[self.offer_item_count as usize] = item;
        self.offer_item_count += 1;
    }

    pub fn remove_offer_item(&mut self, item_index: u8) {
        self.offer_items[item_index as usize] = self.offer_items[self.offer_item_count as usize - 1];
        self.offer_item_count -= 1;
    }

    pub fn accept_offer_item(&mut self, item_index: u8) {
        self.offer_items[0] = self.offer_items[item_index as usize];
        self.offer_item_count = 1;
    }
}


#[account]
#[derive(Default)]
pub struct NftData {
    pub nft_addr: Pubkey,      // 32
    pub owner: Pubkey,      // 32

    // listed nft info for selling
    pub collection_id : u32,    // collection id containing nft
    pub nft_id : u32,   // nft id for selling

    // pool info
    pub pool : Pubkey,
}

#[account]
#[derive(Default)]
pub struct BuyingState {
    pub paid_sol : bool,
    pub paid_nft_count : u8,
    pub paid_nft_account_list: [Pubkey; BUY_MAX_NFT_COUNT],
}

impl BuyingState {
    pub fn add_paid_nft(&mut self, paid_nft_account: Pubkey) {
        self.paid_nft_account_list[self.paid_nft_count as usize] = paid_nft_account;
        self.paid_nft_count += 1;
    }
}

#[account]
#[derive(Default)]
pub struct NftOnSale {
    pub owner : Pubkey,
    pub collection_id : u32,
    pub nft_id : u32,
    pub price : u64,
}
