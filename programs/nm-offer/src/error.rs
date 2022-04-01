use anchor_lang::prelude::*;

#[error]
pub enum NMError {
    #[msg("Invalid Owner")]
    InvalidOwner,

    #[msg("Overflow Offer Count")]
    OverflowOfferCount,

    #[msg("Overflow Token Account Count")]
    OverflowTokenAccountCount,

    #[msg("Invalid Source Account")]
    InvalidSourceAccount,

    #[msg("Nft Not Putted On Sale")]
    NotPuttedOnSale,
}
