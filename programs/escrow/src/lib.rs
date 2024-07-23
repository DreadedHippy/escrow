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

        let creator_balance = ctx.accounts.creator.to_account_info().lamports();

        let total = rent_exempt_lamports + amount;

        if creator_balance < total {
            return Err(CoreErrorCode::InsufficientFunds.into());
        }

        let offer = &mut ctx.accounts.offer;
        offer.creator = *ctx.accounts.creator.key;
        offer.amount = amount;
        offer.accepted = false;
        offer.receiver = None;
        offer.completed = false;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.creator.key(),
            &ctx.accounts.holding_account.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.holding_account.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        let offer = &mut ctx.accounts.offer;

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

        let receiver_key = offer.receiver.unwrap();
        let acc_receiver = ctx.accounts.receiver.key();

        if receiver_key != acc_receiver {
            return Err(CoreErrorCode::ApprovalReceiverKeyNotMatchOfferReceiverKey.into());
        }

        let creator = &ctx.accounts.creator;
        // let receiver_account = &ctx.accounts.receiver;
        let holding_account = &ctx.accounts.holding_account;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.holding_account.to_account_info(),
                    to: ctx.accounts.receiver.to_account_info(),
                },
            ),
            offer.amount,
        )?;

        // let ix = anchor_lang::solana_program::system_instruction::transfer(
        //     &ctx.accounts.holding_account.key(),
        //     &ctx.accounts.receiver.key(),
        //     amount,
        // );
        // anchor_lang::solana_program::program::invoke(
        //     &ix,
        //     &[
        //         ctx.accounts.holding_account.to_account_info(),
        //         ctx.accounts.receiver.to_account_info(),
        //     ],
        // )?;

        offer.completed = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(init, payer = creator, space = Offer::LEN)]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(init, payer = creator, space = HoldingAccount::LEN)]
    pub holding_account: Account<'info, HoldingAccount>,
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
pub struct ApprovePayment<'info> {
    #[account(mut)]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub receiver: SystemAccount<'info>,
    #[account(mut)]
    pub holding_account: Account<'info, HoldingAccount>,
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

impl Offer {
    const LEN: usize = DISCRIMINATOR_LENGTH
        + CREATOR_LENGTH
        + RECEIVER_LENGTH
        + AMOUNT_LENGTH
        + ACCEPTED_LENGTH
        + COMPLETED_LENGTH
        + MAX_DELIVERABLES_LENGTH
        + MAX_CATEGORY_LENGTH
        + MAX_DESCRIPTION_LENGTH;
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
    #[msg("Approval receiver key does not match offer receiver key")]
    ApprovalReceiverKeyNotMatchOfferReceiverKey,
    #[msg("Only the creator of an offer can approve the offer")]
    OnlyOfferCreatorCanApproveOffer,
}

// fn transfer () {
//     let ix = anchor_lang::solana_program::system_instruction::transfer(
//         &ctx.accounts.from.key(),
//         &ctx.accounts.to.key(),
//         amount,
//     );
//     anchor_lang::solana_program::program::invoke(
//         &ix,
//         &[
//             ctx.accounts.from.to_account_info(),
//             ctx.accounts.to.to_account_info(),
//         ],
//     );
// }
