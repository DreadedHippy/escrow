use std::sync::atomic::AtomicBool;

use anchor_lang::prelude::*;

declare_id!("8VewA7Nj7CwWR23UU1goDZGsNf79a7oHq5bsrZCsmATZ");

#[program]
pub mod escrow {

    // use anchor_lang::solana_program::system_program;
    use anchor_lang::system_program;

    use super::CoreErrorCode;
    use super::*;

    pub fn create_offer(ctx: Context<CreateOffer>, amount: u64, offer_id: String) -> Result<()> {
        if offer_id.chars().count() > 50 {
            return Err(CoreErrorCode::OfferIdTooLong.into());
        }
        let offer = &mut ctx.accounts.offer;
        offer.creator = *ctx.accounts.creator.key;
        offer.amount = amount;
        offer.accepted = false;
        offer.receiver = None;
        offer.completed = false;
        offer.withdrawn = false;
        offer.id = offer_id;

        // Before (Not working)
        // ctx.accounts.creator.sub_lamports(amount)?;
        // offer.add_lamports(amount)?;

        // After (Working)
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to: ctx.accounts.offer.to_account_info(),
                },
            ),
            amount,
        )?;
        // let m = ctx

        Ok(())
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        let receiver_key = ctx.accounts.receiver.key();

        let offer = &mut ctx.accounts.offer;

        if offer.accepted {
            return Err(CoreErrorCode::OfferAlreadyAccepted.into());
        }

        if offer.completed {
            return Err(CoreErrorCode::OfferAlreadyCompleted.into());
        }
        // update offer
        offer.accepted = true;
        offer.receiver = Some(receiver_key);

        Ok(())
    }

    pub fn approve_offer_completion(ctx: Context<ApproveOfferCompletion>) -> Result<()> {
        let offer = &mut ctx.accounts.offer;

        if offer.creator != ctx.accounts.creator.key() {
            return Err(CoreErrorCode::OnlyOfferCreatorCanApproveOffer.into());
        }

        if offer.completed == true {
            return Err(CoreErrorCode::OfferAlreadyApproved.into());
        }

        offer.completed = true;

        Ok(())
    }

    pub fn withdraw_offer(ctx: Context<WithdrawOffer>) -> Result<()> {
        let amount = ctx.accounts.offer.amount;

        ctx.accounts.offer.sub_lamports(amount)?;
        ctx.accounts.receiver.add_lamports(amount)?;

        let offer = &mut ctx.accounts.offer;

        offer.withdrawn = true;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64, offer_id: String)]
// #[instruction(amount: u8, offer_id: String, description: String, deliverables: String, category: String)]
pub struct CreateOffer<'info> {
    #[account(
        init,
        payer = creator,
        space = Offer::LEN,
        seeds = [b"offer", creator.key().as_ref(), offer_id.as_bytes()],
        bump
    )]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(
        mut,
        constraint = offer.accepted == false,
        constraint = offer.receiver == None,
        constraint = offer.completed == false
    )]
    pub offer: Account<'info, Offer>,
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveOfferCompletion<'info> {
    #[account(
        mut,
        constraint = offer.creator == *creator.key,
        constraint = offer.completed == false
    )]
    pub offer: Account<'info, Offer>,
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawOffer<'info> {
    #[account(
        mut,
        constraint = offer.completed == true,
        constraint = offer.receiver == Some(*receiver.key),
        constraint = offer.withdrawn == false
    )]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Offer {
    pub creator: Pubkey,
    pub receiver: Option<Pubkey>,
    pub amount: u64,
    pub accepted: bool,
    pub completed: bool,
    pub withdrawn: bool,
    pub id: String,
}

const DISCRIMINATOR_LENGTH: usize = 8;
const CREATOR_LENGTH: usize = 32;
const RECEIVER_LENGTH: usize = 1 + 32;
const AMOUNT_LENGTH: usize = 8;
const ACCEPTED_LENGTH: usize = 1;
const COMPLETED_LENGTH: usize = 1;
const WITHDRAWN_LENGTH: usize = 1;
const MAX_ID_LENGTH: usize = 50 * 4;
// const MAX_DELIVERABLES_LENGTH: usize = 50 * 4;
// const MAX_CATEGORY_LENGTH: usize = 50 * 4;
// const MAX_DESCRIPTION_LENGTH: usize = 240 * 4;

impl Offer {
    const LEN: usize = DISCRIMINATOR_LENGTH
        + CREATOR_LENGTH
        + RECEIVER_LENGTH
        + AMOUNT_LENGTH
        + ACCEPTED_LENGTH
        + COMPLETED_LENGTH
        + WITHDRAWN_LENGTH
        + MAX_ID_LENGTH;
    // + MAX_DELIVERABLES_LENGTH
    // + MAX_CATEGORY_LENGTH
    // + MAX_DESCRIPTION_LENGTH;
}
#[error_code]
pub enum CoreErrorCode {
    #[msg("The offer has not been accepted yet.")]
    OfferNotAccepted,
    #[msg("Category must not exceed 50 characters")]
    CategoryTooLong,
    #[msg("Description must not exceed 240 characters")]
    DescriptionTooLong,
    #[msg("Deliverables must not exceed 50 characters")]
    DeliverablesTooLong,
    #[msg("Insufficient funds to complete transaction")]
    InsufficientFunds,
    #[msg("Offer already accepted")]
    OfferAlreadyAccepted,
    #[msg("Offer already completed")]
    OfferAlreadyCompleted,
    #[msg("Offer already approved")]
    OfferAlreadyApproved,
    #[msg("No receiver attached to offer")]
    NoReceiverAttached,
    #[msg("No receiver key attached to offer")]
    NoReceiverKeyAttached,
    #[msg("Approval receiver key does not match offer receiver key")]
    ApprovalReceiverKeyNotMatchOfferReceiverKey,
    #[msg("Only the creator of an offer can approve the offer")]
    OnlyOfferCreatorCanApproveOffer,
    #[msg("Only approved receiver can receive payment")]
    OnlyApprovedReceiverCanReceivePayment,
    #[msg("Offer id must not exceed 50 characters")]
    OfferIdTooLong,
}
