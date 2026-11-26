# Spark 21

Blackjack on a MagicBlock Ephemeral Rollup with VRF-dealt cards, built for MagicBlock Solana Blitz v10 (Entertainment Finance).
Beat the dealer to 21. Every card is dealt by MagicBlock's verifiable randomness (VRF) and the table account runs on an
Ephemeral Rollup, so hands are provable and fee-free. Chips are virtual and have no cash value. The mascot Spark is an original
drawing, not an official MagicBlock asset.

- Live demo: https://jarvis-184.github.io/spark21/
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

## Why MagicBlock
- **VRF**: every card (deal, hit, stand, double) comes from a MagicBlock VRF value delivered in an oracle callback, so the player does not choose the cards.
- **Ephemeral Rollup**: the player table account is delegated to the rollup, so each action confirms fast and without fees. `undelegate` then commits the final state back to Solana (chips 1500 verified on the base chain).

## The 12 recorded hands (Solana devnet)
Rollup transactions open in the explorer with the MagicBlock devnet validator as custom RPC. Totals are the hand values at the end of each hand.

| # | Bet | You | Dealer | Result | Chips after | Transactions (rollup explorer) |
|---|---|---|---|---|---|---|
| 1 | 100 | 20 | 25 | WIN | 1100 | [deal](https://explorer.solana.com/tx/Ppno8kRUFNDTK73R8o8oSLXgHtLeWFUcmJKfF2kuRX3LhPKT5PAP5giZ4GYN2oYTxkEnFG2TsH5a2Sw4CMnZXnN?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/5BzUa5ieEgJkVxgyB7SfyrjEYCY9CAHVzKW5gTJRjVvmVZZ5kxNmWCT9CYpMRNPmqYiv6BKyMd2uBubnDAKNWon5?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 2 | 100 | 21 | 18 | WIN | 1220 | [deal](https://explorer.solana.com/tx/2irTMgMSnXT6GUni431VHJACYrdTYpmMxUCJTJESBBsxJQ8yKMJTYed9Tiikzar66SPHjcY82FzgWGfpeshdk99f?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/5ojTWJhftJubVtjRprQ6HMhHdoR5DtLBwury858Y7VFrttGZekMBZHKqVQUAUPNDDAEXUxufjbqpozT8GJxvqTBn?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/3UMA7KDzHe1af6VhngU7bMnumCASS4sfrXeCkuKv4K4xPqnEy1HvmkyJjtCBWWaRpmRQ9hwqay8NgzUxzcpAMKPz?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 3 | 100 | 13 | 21 | LOSE | 1120 | [deal](https://explorer.solana.com/tx/3PNh8Qkdv6qcNDuhSu26r5QB1Ds2LuLsPtk8a6uAVtsXmev2baSXFLPDjbxAXd7QUbFyYBtqv2dmZQ642ASwUk9y?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/3SMswcRpxkUR9rEzcKtnzjoB9kEw4RpzxS3V6iUPGQ9uoJxFZCDAKw5sTC5Xy8D4g3x4WBKYwKd1NeJb2jm15tWf?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 4 | 100 | 21 | 19 | WIN | 1220 | [deal](https://explorer.solana.com/tx/4kCQKhPU3e3Fbjgxx6gJvs6Aw9ZsppxugNEyJcM3hHUNVdcEQ3MBSpdwdxTXirk7MrSBkCofH4MaijeDCfckPc1Y?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/5sK249hGYEvawVCd3ZEhu8o9PBSwrt9VJumWAVaLHQ4AXZYggdaP6UzGY47yRqsEgcLeXefWWsj9EwrkHmiyh1ar?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/2K3YvDR8WP4sPqFrBdiEYm7Tw7FvBefa1ChazaWHHJ8uGT9CtDCZqxXp2z61fyo9oo9AqLP7grj1zXRazXMQz1vp?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 5 | 100 | 18 | 20 | LOSE | 1120 | [deal](https://explorer.solana.com/tx/61Z36gSiAfV7QXALtGTEWz9gygnfRLRdTMbzpYShTkN5X5jTkWDhpNEWUpbS4wUZ4poQTr97zsDZAWCX21xCVFvP?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/3undX4rPRckMCqPZ6Hqc3jfWAV7mt7E59u8mg7X1GYisECC7LRA9XG9D7tnYxaDxURKF27qXGnKZbLodbkgW5mQQ?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 6 | 100 | 13 | 24 | WIN | 1220 | [deal](https://explorer.solana.com/tx/3ADJ37LUZECYRuDUMP32FZufhR2vopmY4y8tcbHuYnPatfkkir3XkKZZSpqzoZ5mzJXJNrBExxAMdd5roefyftVR?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/5tT6zqg8mdNigEaq9GpdMJN727hPR3xBjp1WXW21ebwAUiab52S5yZPduADWtY3Nk9FPjCg4xjDHoLXzsQD61nEv?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 7 | 100 | 20 | 24 | WIN | 1340 | [deal](https://explorer.solana.com/tx/Bmwy6HYoEptPzHZgVRAbSNRbBRT2ZFFX8adEa4cgfCXFhsJYGkX2t3EUcyjfmQeZCbvDNEaQdPJgqv8UVrvnd3L?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/58mhRaVjeH8Rjtb1Sn1iPxMmKpJzrFsuoNJa3m8sYfk8MyAMm2MMC5romnjHJ1KZz5m66Ygdz1PtaF2u1X7HTRBP?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/DHaM7KyAXFn3HkY49Xhv6moW1D9GU8sUtUEWfNRz7G2WzWSuTpvuxUbAc75yZjVBkv5qF6F7bqN6Y9UMGTddNF9?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 8 | 100 | 17 | 24 | WIN | 1480 | [deal](https://explorer.solana.com/tx/2M1iRGCCMDSLqQdwdvADXjMXo5tgXih5o1rd1wPL5CV1zHhsj9uvFfTpGCBDvp8hEa8pKSMkECusqsQnosdzMVVC?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/4r4qRUhWnJe7bx13ncHW2YpFBdY57wL1T1wZmNCX8GTBaVKbco4dHbepz95e1H7LPrLe3WbCcAsyaWYjKKt449Dp?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/5pWuWJRAUA76iuoPjKpZ57gm2r6PQzwVe7d1hEoDCQM3fZs9gjx6DY5TgNryqnJ9Dr8C38ABtBvufWm9iT6cyhnj?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 9 | 100 | 18 | 21 | LOSE | 1380 | [deal](https://explorer.solana.com/tx/2uFD3tfczozhnoG1HCUU47Kf8aRKnHNnSPZR1JPbV49pt3ZLahTmjoQb3aMUJNxrrUUwSZaUwF7JtBJwAipXEdz2?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/3HibGffg61vCqhkPg5zyy5f13yL6ww2ZDxqcU1DqfNHbEYCZX9aNm3jfVCHDHiRQ6GTvpTUiELjiAuP8iNDMM8GC?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/4BSdBCoUDJNaZMKTPPm194yHN1fsorHQY1t5tYsmfQ2f7rmc7mWfsokRbPFPoy6GCo1y8bfPBY79YV25LgVtiKrK?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 10 | 100 | 20 | 23 | WIN | 1480 | [deal](https://explorer.solana.com/tx/3Ycw7hoafbPGKuwnUKdF1nFowH915VWyonNrBw4w8DxWpns76MXNshm6Kgu1Q6mhPgHi6mjPbuYeqntEgxtfk2p7?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/2JaVzV3ubU9Cvgo9oCn4HxMruitE8F7YKaXPiThZuLcJogVpmzj5ypWTTdbemaDKtvW22Pb93YHVjrutwdEBbyK2?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 11 | 100 | 21 | 20 | WIN | 1600 | [deal](https://explorer.solana.com/tx/5ZWNBK6Z7JHh7XvQBpeqeEpSjRYn7m6rGovgs6JGnZxBfJp5z8HMweYdUrJCrsyzhg8cpriUXGZAmLBN8LJi8hNa?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/XZqq4uWTYrPS7ssYhYD3B5VZk9vFHhpBcSJVVBitncc67uPNHHvENZX1LgL4WNGqcn7kyBWxKU4mH9g3Ux7gmdM?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/BCfpJ1Gxi74bEoFmKaMBspzNXBxN5fcuYihkLxPdF3gUDKKGdDTpkThYa9rJrKBQV4J1uSixZLpwAJa7kQbWEVK?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |
| 12 | 100 | 17 | 20 | LOSE | 1500 | [deal](https://explorer.solana.com/tx/3yZ1uBxN3jBXRuhkbuBo6ZJT4Lfzhk95ST1zjJXBWVuwjj2yNadezPZTQchLCcpKMREMrDxi4ZSWGmyFD3XnnVEE?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [hit](https://explorer.solana.com/tx/2GnGUyxzWpmiBJq7482UuHNj66pS52Eehaq4KZyigHoHwTLb3uMSppFzr9zHg4NPzp1LF3WJ4S1NeB4aVNPx7V8Z?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app), [stand](https://explorer.solana.com/tx/4e1dwujE156H4rdvCSjiya36d3PMbVY2e3ji8SXDNgbVBtSwYm43jEyQL9RVw1KbzsNkeAp73b1RAnKibD9bvhqb?cluster=custom&customUrl=https%3A%2F%2Fdevnet-us.magicblock.app) |

## Honest limits
- Cards come from an unlimited shoe: every draw is independent, so the same card can appear twice in a hand.
- The account is public, so the dealer's hole card can be read on-chain by someone who looks. This is a hackathon prototype, not a gambling product.
- The web demo's music, fireworks, levels, badges, daily spin, goals and feed are client-side and stored in localStorage. The Replay tab plays back recorded on-chain hands; it is not a live connection.
- No timeout or refund: if the VRF callback never arrives, the table stays "pending", and once chips reach 0 there is no refill on-chain (the web Practice mode has a daily spin instead).
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
