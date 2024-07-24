use std::sync::atomic::AtomicBool;

use anchor_lang::prelude::*;

declare_id!("8VewA7Nj7CwWR23UU1goDZGsNf79a7oHq5bsrZCsmATZ");

#[program]
pub mod escrow {

    use anchor_lang::system_program;

    use super::CoreErrorCode;
    use super::*;

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        amount: u64,
        offer_id: String,
        description: String,
        deliverables: String,
        category: String,
    ) -> Result<()> {
        if description.chars().count() > 240 {
            return Err(CoreErrorCode::DescriptionTooLong.into());
        }

        if category.chars().count() > 50 {
            return Err(CoreErrorCode::CategoryTooLong.into());
        }

        if deliverables.chars().count() > 50 {
            return Err(CoreErrorCode::DeliverablesTooLong.into());
        }
        let rent_exempt_lamports = Rent::get()?.minimum_balance(Offer::LEN);

        // Calculate total amount required to store the sol and be rent exempt;

        let creator_balance = ctx.accounts.creator.to_account_info().lamports();

        let total = rent_exempt_lamports + amount;

        if creator_balance < total {
            // Throw error if insufficient balance
            return Err(CoreErrorCode::InsufficientFunds.into());
        }

        // Initialize the offer
        let offer = &mut ctx.accounts.offer;
        offer.creator = *ctx.accounts.creator.key;
        offer.amount = amount;
        offer.accepted = false;
        offer.receiver = None;
        offer.completed = false;
        offer.payment_received = false;

        // Tranfer from the creator's account into the `Offer` PDA
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.offer.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, total)?;

        Ok(())
    }

    /// When a `receiver` accepts to undertake an offer
    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        let offer = &mut ctx.accounts.offer;

        msg!(
            "Ammount inside the offer account after {}",
            offer.to_account_info().lamports()
        );

        if offer.accepted {
            return Err(CoreErrorCode::OfferAlreadyAccepted.into());
        }

        if offer.completed {
            return Err(CoreErrorCode::OfferAlreadyCompleted.into());
        }

        offer.accepted = true;
        offer.receiver = Some(ctx.accounts.receiver.key());
        Ok(())
    }

    /// Creator of an offer approves that receiver has completed said offer
    pub fn approve_payment(ctx: Context<ApprovePayment>) -> Result<()> {
        let offer = &mut ctx.accounts.offer;
        let amount = offer.amount;

        if offer.creator != ctx.accounts.creator.key() {
            return Err(CoreErrorCode::OnlyOfferCreatorCanApproveOffer.into());
        }

        if !offer.accepted {
            return Err(CoreErrorCode::OfferNotAccepted.into());
        }

        if offer.receiver.is_none() {
            return Err(CoreErrorCode::NoReceiverAttached.into());
        }

        offer.completed = true;

        Ok(())
    }

    // Here is where the issue lies
    /// Receiver wants to receive payment into their wallet
    pub fn receive_payment(ctx: Context<ReceivePayment>) -> Result<()> {
        // let offer = &mut ctx.acccounts.offer;

        if ctx.accounts.offer.receiver.is_none() {
            return Err(CoreErrorCode::NoReceiverAttached.into());
        }

        // let receiver = offer.receiver_bump.unwrap();
        let amount = ctx.accounts.offer.amount;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.offer.to_account_info(),
                to: ctx.accounts.receiver.to_account_info(),
            },
        );

        system_program::transfer(cpi_context, amount)?;

        ctx.accounts.offer.payment_received = true;

        Ok(())
    }
}

#[derive(Accounts)]
// #[instruction(amount: u8, offer_id: String, description: String, deliverables: String, category: String)]
pub struct CreateOffer<'info> {
    #[account(init, payer = creator, space = Offer::LEN, seeds = [b"offer", creator.key().as_ref()], bump)]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetOffer<'info> {
    #[account()]
    pub offer: Account<'info, Offer>,
}

#[derive(Accounts)]
pub struct ReceivePayment<'info> {
    #[account(mut,
        constraint = offer.receiver == Some(receiver.key()),
        constraint = offer.accepted == true,
        constraint = offer.completed == true
    )]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApprovePayment<'info> {
    #[account(mut, constraint = offer.creator == creator.key())]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Offer {
    pub creator: Pubkey,
    pub receiver: Option<Pubkey>,
    pub amount: u64,
    pub accepted: bool,
    pub completed: bool,
    pub deliverables: String,
    pub category: String,
    pub desription: String,
    pub payment_received: bool,
}

const DISCRIMINATOR_LENGTH: usize = 8;
const CREATOR_LENGTH: usize = 32;
const RECEIVER_LENGTH: usize = 1 + 32;
const AMOUNT_LENGTH: usize = 8;
const ACCEPTED_LENGTH: usize = 1;
const COMPLETED_LENGTH: usize = 1;
const MAX_DELIVERABLES_LENGTH: usize = 50 * 4;
const MAX_CATEGORY_LENGTH: usize = 50 * 4;
const MAX_DESCRIPTION_LENGTH: usize = 240 * 4;
const BUMP_LENGTH: usize = 1;
const PAYMENT_RECEIVED_LENGTH: usize = 1;

impl Offer {
    const LEN: usize = DISCRIMINATOR_LENGTH
        + CREATOR_LENGTH
        + RECEIVER_LENGTH
        + AMOUNT_LENGTH
        + ACCEPTED_LENGTH
        + COMPLETED_LENGTH
        + MAX_DELIVERABLES_LENGTH
        + MAX_CATEGORY_LENGTH
        + MAX_DESCRIPTION_LENGTH
        + BUMP_LENGTH
        + PAYMENT_RECEIVED_LENGTH;
}

#[account]
pub struct HoldingAccount {
    pub data: u64,
}

impl HoldingAccount {
    const LEN: usize = 8 + 8;
}

#[account]
pub struct ReceiverAccount {
    pub is_initialized: bool,
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
}
