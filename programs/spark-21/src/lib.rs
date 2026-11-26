//! Spark 21: blackjack on a MagicBlock Ephemeral Rollup.
//!
//! Every card comes from MagicBlock's verifiable randomness (VRF). Each action (deal, hit, stand,
//! double) requests a fresh random value, and the oracle callback applies the action on the player
//! account, which is delegated to an Ephemeral Rollup so play is fast and fee-free.
//! Cards are drawn from an unlimited shoe (every draw is independent). Chips are virtual.
//! The account is public, so a determined reader can see the dealer's hole card on-chain; this is
//! a hackathon prototype, not a gambling product. Wiring follows MagicBlock's public examples.
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{
    anchor::{commit, delegate, ephemeral, vrf, vrf_callback},
    cpi::DelegateConfig,
    ephem::MagicIntentBundleBuilder,
    vrf::{
        instructions::{create_request_scoped_randomness_ix, RequestRandomnessParams},
        types::SerializableAccountMeta,
    },
};

declare_id!("4C38DbUP5fgkkn8NTLXqtskoEcW5r6tuMVgDgFWqMEnu");

pub const SEED: &[u8] = b"spark21";
pub const START_CHIPS: u64 = 1_000;
pub const ACT_DEAL: u8 = 0;
pub const ACT_HIT: u8 = 1;
pub const ACT_STAND: u8 = 2;
pub const ACT_DOUBLE: u8 = 3;
pub const R_NONE: u8 = 0;
pub const R_WIN: u8 = 1;
pub const R_LOSE: u8 = 2;
pub const R_PUSH: u8 = 3;
pub const R_BLACKJACK: u8 = 4;
pub const R_BUST: u8 = 5;
pub const MAX_CARDS: usize = 12;

fn card_value_total(cards: &[u8]) -> u8 {
    let mut total: u32 = 0;
    let mut aces = 0;
    for c in cards {
        let r = c % 13;
        if r == 0 {
            aces += 1;
            total += 11;
        } else {
            total += core::cmp::min(r as u32 + 1, 10);
        }
    }
    while total > 21 && aces > 0 {
        total -= 10;
        aces -= 1;
    }
    total as u8
}

/// Draws cards 0..51 from the VRF bytes (rejection sampling avoids modulo bias).
struct Shoe {
    bytes: [u8; 32],
    pos: usize,
}
impl Shoe {
    fn draw(&mut self) -> u8 {
        while self.pos < 32 {
            let b = self.bytes[self.pos];
            self.pos += 1;
            if b < 208 {
                return b % 52;
            }
        }
        // 32 random bytes are far more than a hand needs; if we ever run out, stretch deterministically.
        let mut h = [0u8; 32];
        for i in 0..32 {
            h[i] = self.bytes[i].wrapping_mul(31).wrapping_add(i as u8 + 7);
        }
        self.bytes = h;
        self.pos = 0;
        self.draw()
    }
}

#[ephemeral]
#[program]
pub mod spark_21 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let t = &mut ctx.accounts.table;
        if t.hands == 0 && t.chips == 0 {
            t.owner = ctx.accounts.payer.key();
            t.chips = START_CHIPS;
        }
        Ok(())
    }

    /// action: 0 deal (needs bet), 1 hit, 2 stand, 3 double. Requests a VRF value; the callback applies it.
    pub fn act(ctx: Context<Act>, action: u8, bet: u64, client_seed: u8) -> Result<()> {
        let t = &mut ctx.accounts.table;
        require!(!t.pending, SparkError::Pending);
        match action {
            ACT_DEAL => {
                require!(t.phase == 0, SparkError::HandInProgress);
                require!(bet > 0 && bet <= t.chips, SparkError::BadBet);
                t.chips -= bet;
                t.bet = bet;
                t.result = R_NONE;
                t.last_gain = 0;
                t.pc = 0;
                t.dc = 0;
            }
            ACT_HIT | ACT_STAND => require!(t.phase == 1, SparkError::NoHand),
            ACT_DOUBLE => {
                require!(t.phase == 1 && t.pc == 2, SparkError::CannotDouble);
                require!(t.chips >= t.bet, SparkError::NotEnoughChips);
                t.chips -= t.bet;
                t.bet = t.bet.saturating_mul(2);
            }
            _ => return err!(SparkError::BadAction),
        }
        t.pending = true;
        t.pending_action = action;
        let ix = create_request_scoped_randomness_ix(RequestRandomnessParams {
            payer: ctx.accounts.payer.key(),
            oracle_queue: ctx.accounts.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: instruction::CallbackAct::DISCRIMINATOR.to_vec(),
            caller_seed: [client_seed; 32],
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: ctx.accounts.table.key(),
                is_signer: false,
                is_writable: true,
            }]),
            callback_args: None,
            ..Default::default()
        });
        ctx.accounts
            .invoke_signed_vrf(&ctx.accounts.payer.to_account_info(), &ix)?;
        Ok(())
    }

    /// Called by the VRF oracle. Deals or draws cards from the random bytes and settles when the hand ends.
    pub fn callback_act(ctx: Context<CallbackAct>, randomness: [u8; 32]) -> Result<()> {
        let t = &mut ctx.accounts.table;
        require!(t.pending, SparkError::NoPending);
        let mut shoe = Shoe { bytes: randomness, pos: 0 };
        let action = t.pending_action;
        t.pending = false;
        match action {
            ACT_DEAL => {
                let p0 = shoe.draw();
                let d0 = shoe.draw();
                let p1 = shoe.draw();
                let d1 = shoe.draw();
                t.player = [0; MAX_CARDS];
                t.dealer = [0; MAX_CARDS];
                t.player[0] = p0;
                t.player[1] = p1;
                t.dealer[0] = d0;
                t.dealer[1] = d1;
                t.pc = 2;
                t.dc = 2;
                t.phase = 1;
                let pb = card_value_total(&t.player[..2]) == 21;
                let db = card_value_total(&t.dealer[..2]) == 21;
                if pb || db {
                    settle(t, pb, db);
                }
            }
            ACT_HIT => {
                let c = shoe.draw();
                let n = t.pc as usize;
                t.player[n] = c;
                t.pc += 1;
                if card_value_total(&t.player[..t.pc as usize]) > 21 || t.pc as usize >= MAX_CARDS - 1 {
                    settle(t, false, false);
                }
            }
            ACT_STAND => {
                dealer_play(t, &mut shoe);
                settle(t, false, false);
            }
            ACT_DOUBLE => {
                let c = shoe.draw();
                let n = t.pc as usize;
                t.player[n] = c;
                t.pc += 1;
                if card_value_total(&t.player[..t.pc as usize]) <= 21 {
                    dealer_play(t, &mut shoe);
                }
                settle(t, false, false);
            }
            _ => {}
        }
        msg!("action={} chips={} result={} pc={} dc={}", action, t.chips, t.result, t.pc, t.dc);
        Ok(())
    }

    pub fn delegate(ctx: Context<DelegateInput>) -> Result<()> {
        ctx.accounts.delegate_table(
            &ctx.accounts.user,
            &[SEED, &ctx.accounts.user.key().to_bytes().as_slice()],
            DelegateConfig {
                validator: ctx.remaining_accounts.first().map(|acc| acc.key()),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        MagicIntentBundleBuilder::new(
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.magic_context.to_account_info(),
            ctx.accounts.magic_program.to_account_info(),
        )
        .commit_and_undelegate(&[ctx.accounts.table.to_account_info()])
        .build_and_invoke()?;
        Ok(())
    }
}

fn dealer_play(t: &mut Table, shoe: &mut Shoe) {
    while card_value_total(&t.dealer[..t.dc as usize]) < 17 && (t.dc as usize) < MAX_CARDS - 1 {
        let c = shoe.draw();
        let n = t.dc as usize;
        t.dealer[n] = c;
        t.dc += 1;
    }
}

/// Settle the hand. pb/db flag natural blackjacks found at the deal.
fn settle(t: &mut Table, pb: bool, db: bool) {
    let pv = card_value_total(&t.player[..t.pc as usize]);
    let dv = card_value_total(&t.dealer[..t.dc as usize]);
    let combo = 100 + 10 * (t.streak.min(5) as u64);
    let bet = t.bet;
    let (res, gain) = if pv > 21 {
        (R_BUST, 0)
    } else if pb && !db {
        (R_BLACKJACK, bet.saturating_mul(250).saturating_mul(combo) / 10_000)
    } else if db && !pb {
        (R_LOSE, 0)
    } else if pb && db {
        (R_PUSH, bet)
    } else if dv > 21 || pv > dv {
        (R_WIN, bet.saturating_mul(2).saturating_mul(combo) / 100)
    } else if pv == dv {
        (R_PUSH, bet)
    } else {
        (R_LOSE, 0)
    };
    t.chips = t.chips.saturating_add(gain);
    t.last_gain = gain;
    t.result = res;
    t.hands = t.hands.saturating_add(1);
    if res == R_WIN || res == R_BLACKJACK {
        t.wins = t.wins.saturating_add(1);
        t.streak = t.streak.saturating_add(1);
        if t.streak > t.best_streak {
            t.best_streak = t.streak;
        }
    } else if res != R_PUSH {
        t.streak = 0;
    }
    t.phase = 0;
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, payer = payer, space = 8 + Table::SIZE, seeds = [SEED, payer.key().to_bytes().as_slice()], bump)]
    pub table: Account<'info, Table>,
    pub system_program: Program<'info, System>,
}

#[vrf]
#[derive(Accounts)]
pub struct Act<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [SEED, payer.key().to_bytes().as_slice()], bump)]
    pub table: Account<'info, Table>,
    /// CHECK: validated by the ephemeral VRF program when it processes the request
    #[account(mut)]
    pub oracle_queue: UncheckedAccount<'info>,
}

#[vrf_callback]
#[derive(Accounts)]
pub struct CallbackAct<'info> {
    #[account(mut)]
    pub table: Account<'info, Table>,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: the PDA to delegate
    #[account(mut, del, seeds = [SEED, user.key().to_bytes().as_slice()], bump)]
    pub table: UncheckedAccount<'info>,
}

#[commit]
#[derive(Accounts)]
pub struct Undelegate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [SEED, payer.key().to_bytes().as_slice()], bump)]
    pub table: Account<'info, Table>,
}

#[account]
pub struct Table {
    pub owner: Pubkey,
    pub chips: u64,
    pub bet: u64,
    pub last_gain: u64,
    pub hands: u32,
    pub wins: u32,
    pub streak: u16,
    pub best_streak: u16,
    pub phase: u8,
    pub pending: bool,
    pub pending_action: u8,
    pub pc: u8,
    pub dc: u8,
    pub result: u8,
    pub player: [u8; MAX_CARDS],
    pub dealer: [u8; MAX_CARDS],
}
impl Table {
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 4 + 4 + 2 + 2 + 1 + 1 + 1 + 1 + 1 + 1 + MAX_CARDS + MAX_CARDS;
}

#[error_code]
pub enum SparkError {
    #[msg("a request is still pending")]
    Pending,
    #[msg("a hand is already in progress")]
    HandInProgress,
    #[msg("no hand in progress")]
    NoHand,
    #[msg("bad bet")]
    BadBet,
    #[msg("double is only allowed on the first two cards")]
    CannotDouble,
    #[msg("not enough chips")]
    NotEnoughChips,
    #[msg("unknown action")]
    BadAction,
    #[msg("no pending request to settle")]
    NoPending,
}
