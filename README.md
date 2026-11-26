# Spark 21

Blackjack on a MagicBlock Ephemeral Rollup with VRF-dealt cards, built for MagicBlock Solana Blitz v10 (Entertainment Finance).
Beat the dealer to 21. Every card is dealt by MagicBlock's verifiable randomness (VRF) and the table account runs on an
Ephemeral Rollup, so hands are provable and fee-free. Chips are virtual and have no cash value. The mascot Spark is an original
drawing, not an official MagicBlock asset.

- Program (Solana devnet): `4C38DbUP5fgkkn8NTLXqtskoEcW5r6tuMVgDgFWqMEnu`
- Recorded run on devnet: 12 real hands, 31 actions (deal, hit, stand, double), 8 wins and 4 losses, 1000 chips to 1500.
  Signatures and cards are in `docs/hands.json`. Each hand was re-checked off-chain against the rules (12 of 12 matched).
- Demo: `docs/index.html` (Replay tab = real devnet hands; Practice tab = local shuffle, clearly labeled).

## How it works
One PDA per player (`seed "spark21" + wallet`) holds chips, the bet and both hands.
- `initialize` creates the table with 1000 chips. `delegate` moves it to an Ephemeral Rollup. `undelegate` commits state back to Solana.
- `act(action, bet, client_seed)` with action 0 deal, 1 hit, 2 stand, 3 double requests a fresh VRF value.
- `callback_act(randomness)` (called by the VRF program) deals or draws cards from the random bytes, plays the dealer (stands on 17) and settles the hand.
- Payouts: win pays 2x the bet, blackjack 2.5x, push returns the bet. A win streak adds +10% per previous consecutive win (max +50%) to winning payouts.

## Honest limits
- Cards come from an unlimited shoe: every draw is independent, so the same card can appear twice in a hand.
- The account is public, so the dealer's hole card can be read on-chain by someone who looks. This is a hackathon prototype, not a gambling product.
- The web demo's music, fireworks, levels, badges, daily spin, goals and feed are client-side and stored in localStorage. The Replay tab plays back recorded on-chain hands; it is not a live connection.
- Only one wallet played the recorded run. No wash trades or fake users.

## Build and run
```
cargo build-sbf
solana program deploy target/deploy/spark_21.so --program-id <keypair> --url devnet
cd client && npm i && HANDS=12 node play-spark.mjs init,delegate,play   # then: node play-spark.mjs undelegate
```

## Credits
Delegation and VRF wiring follow MagicBlock's public examples (https://github.com/magicblock-labs/magicblock-engine-examples).
Game rules, program, client and web page are ours.
