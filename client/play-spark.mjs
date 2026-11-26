// Spark 21 client: init -> delegate -> play N hands (basic strategy) with VRF-dealt cards -> undelegate.
import anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { ConnectionMagicRouter } from "@magicblock-labs/ephemeral-rollups-sdk";
import fs from "fs";
const { AnchorProvider, Program, Wallet, BN } = anchor;
const idl = JSON.parse(fs.readFileSync(new URL("./idl.spark.json", import.meta.url)));
const PROGRAM_ID = new PublicKey(idl.address);
const ORACLE_QUEUE = new PublicKey("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc");
const kp = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(process.env.KEYPAIR || process.env.HOME + "/.config/solana/id.json"))));
const base = new Connection("https://api.devnet.solana.com", "confirmed");
const router = new ConnectionMagicRouter("https://devnet-router.magicblock.app", { wsEndpoint: "wss://devnet-router.magicblock.app" });
const steps = (process.argv[2] || "all").split(",");
const HANDS = Number(process.env.HANDS || 10);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const [tablePda] = PublicKey.findProgramAddressSync([Buffer.from("spark21"), kp.publicKey.toBuffer()], PROGRAM_ID);
const mk = (c) => new Program(idl, new AnchorProvider(c, new Wallet(kp), { commitment: "confirmed", skipPreflight: true }));
const baseProg = mk(base);
const RANK = ["A","2","3","4","5","6","7","8","9","10","J","Q","K"], SUIT = ["S","H","D","C"];
const name = (c) => RANK[c % 13] + SUIT[Math.floor(c / 13)];
function total(cs) { let t = 0, a = 0; for (const c of cs) { const r = c % 13; if (r === 0) { a++; t += 11; } else t += Math.min(r + 1, 10); } while (t > 21 && a > 0) { t -= 10; a--; } return t; }
const RES = ["none","WIN","LOSE","PUSH","BLACKJACK","BUST"];
const REC = [];
async function run() {
  if (steps.includes("all") || steps.includes("init")) console.log("initialize", await baseProg.methods.initialize().accounts({ payer: kp.publicKey }).rpc());
  const status = await router.getDelegationStatus(tablePda).catch(() => null);
  if ((steps.includes("all") || steps.includes("delegate")) && (!status || !status.isDelegated)) {
    const v = await router.getClosestValidator();
    console.log("validator", v.identity, v.fqdn);
    console.log("delegate", await baseProg.methods.delegate().accounts({ user: kp.publicKey }).remainingAccounts([{ pubkey: new PublicKey(v.identity), isSigner: false, isWritable: false }]).rpc({ skipPreflight: true }));
    fs.writeFileSync("validator.spark.json", JSON.stringify(v));
    await sleep(4000);
  }
  const v = JSON.parse(fs.readFileSync("validator.spark.json"));
  const eph = new Connection(v.fqdn, { wsEndpoint: v.fqdn.replace("https", "wss"), commitment: "confirmed" });
  const prog = mk(eph);
  const fetchT = () => prog.account.table.fetch(tablePda, "processed");
  let seed = Date.now() % 200;
  async function act(action, bet) {
    const before = await fetchT();
    const sig = await prog.methods.act(action, new BN(bet), (seed++) % 250).accounts({ payer: kp.publicKey, table: tablePda, oracleQueue: ORACLE_QUEUE }).rpc({ skipPreflight: true });
    for (let k = 0; k < 60; k++) { await sleep(400); const t = await fetchT(); if (!t.pending) return { sig, t, before }; }
    throw new Error("timeout waiting for VRF callback");
  }
  if (steps.includes("all") || steps.includes("play")) {
    for (let h = 0; h < HANDS; h++) {
      let t0 = await fetchT();
      const bet = Math.max(1, Math.min(100, Number(t0.chips)));
      const sigs = [];
      const start = Date.now();
      let r = await act(0, bet); sigs.push(["deal", r.sig]); let t = r.t;
      const startChips = Number(r.before.chips);
      while (t.phase === 1) {
        const pc = Array.from(t.player).slice(0, t.pc), dc = Array.from(t.dealer).slice(0, t.dc);
        const pv = total(pc), up = Math.min((dc[0] % 13) + 1, 10) === 1 ? 11 : Math.min((dc[0] % 13) + 1, 10);
        let a;
        if (pc.length === 2 && (pv === 10 || pv === 11) && up < 10 && Number(t.chips) >= Number(t.bet)) a = 3;
        else if (pv >= 17) a = 2;
        else if (pv >= 12 && up <= 6) a = 2;
        else a = 1;
        r = await act(a, 0); sigs.push([["", "hit", "stand", "double"][a], r.sig]); t = r.t;
      }
      const pc = Array.from(t.player).slice(0, t.pc), dc = Array.from(t.dealer).slice(0, t.dc);
      // independent check of the on-chain settlement
      const pv = total(pc), dv = total(dc), bj = (c) => c.length === 2 && total(c) === 21;
      let exp;
      if (pv > 21) exp = "BUST"; else if (bj(pc) && !bj(dc)) exp = "BLACKJACK"; else if (bj(dc) && !bj(pc)) exp = "LOSE"; else if (bj(pc) && bj(dc)) exp = "PUSH"; else if (dv > 21 || pv > dv) exp = "WIN"; else if (pv === dv) exp = "PUSH"; else exp = "LOSE";
      const ok = exp === RES[t.result] && (pv > 21 || bj(pc) || bj(dc) || dv >= 17);
      console.log(`hand ${h + 1}: bet ${Number(t.bet)} you [${pc.map(name)}]=${pv} dealer [${dc.map(name)}]=${dv} -> ${RES[t.result]} gain ${Number(t.lastGain)} chips ${startChips}->${Number(t.chips)} actions ${sigs.map((s) => s[0]).join(",")} ${ok ? "CHECK-OK" : "CHECK-MISMATCH expected " + exp}`);
      REC.push({ n: h + 1, bet: Number(t.bet), player: pc, dealer: dc, pv, dv, result: RES[t.result], gain: Number(t.lastGain), chipsBefore: startChips, chipsAfter: Number(t.chips), streakAfter: t.streak, actions: sigs, ms: Date.now() - start, check: ok });
      fs.writeFileSync("hands.out.json", JSON.stringify({ program: PROGRAM_ID.toBase58(), table: tablePda.toBase58(), validator: v.fqdn, hands: REC }, null, 1));
      if (Number(t.chips) <= 0) break;
    }
    const f = await fetchT(); console.log("final", `chips=${f.chips} hands=${f.hands} wins=${f.wins} best=${f.bestStreak}`);
  }
  if (steps.includes("undelegate")) {
    console.log("undelegate", await prog.methods.undelegate().accounts({ payer: kp.publicKey }).rpc({ skipPreflight: true }));
    await sleep(5000);
    const a = await baseProg.account.table.fetch(tablePda); console.log("base after commit", `chips=${a.chips} hands=${a.hands} wins=${a.wins}`);
  }
}
run().catch((e) => { console.error(e); process.exit(1); });
