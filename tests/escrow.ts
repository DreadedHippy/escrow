import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { Escrow } from "../target/types/escrow";

const { SystemProgram } = anchor.web3;


// this airdrops sol to an address
async function airdropSol(publicKey, amount) {
  let airdropTx = await anchor.getProvider().connection.requestAirdrop(publicKey, amount);
  await confirmTransaction(airdropTx);
}

async function confirmTransaction(tx) {
  const latestBlockHash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: tx,
  });
}
const LAMPORTS_PER_SOL = 1000000000
describe("escrow", async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const creator = anchor.web3.Keypair.generate();

  // create a new keypair
  const receiver = anchor.web3.Keypair.generate();

  // Generate a new keypair to create accounts owned by our program
  const programOwnedAccount = anchor.web3.Keypair.generate();

  const offerAmount = 100 * LAMPORTS_PER_SOL;


  // Airdrop sol to the creator account;

  it("Creates an offer", async () => {
    console.log("---------Creating an offer...-------");
    await airdropSol(creator.publicKey, (1e9)* 1000); // 1000 sol
    const offerId = "offer1";
    const[offerPDA, _] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("offer"), creator.publicKey.toBuffer(), Buffer.from(offerId)], program.programId);

    
    await getBalances(creator.publicKey, offerPDA, "Beginning");

    let amount = new anchor.BN(offerAmount);
    
    await program.methods.createOffer(amount, offerId)
    .accounts({
      creator: creator.publicKey,
      offer: offerPDA
    })
    .signers([creator])
    .rpc();


    await getBalances(creator.publicKey, offerPDA, "After offer created");


    // check balance and assert that it is equal to amount
    const offer_check = await program.account.offer.fetch(offerPDA);
    assert.ok(offer_check.amount.eq(amount));
  });

  it ("Accepts an offer", async () => {
    console.log("---------Accepting an offer...-------");

    const offerId = "offer1";
    const[offerPDA, _] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("offer"), creator.publicKey.toBuffer(), Buffer.from(offerId)], program.programId);

    await program.methods.acceptOffer()
    .accounts({
      offer: offerPDA,
      receiver: receiver.publicKey
    })
    .signers([receiver])
    .rpc();


    const offer_check = await program.account.offer.fetch(offerPDA);
    assert.ok(offer_check.accepted);
    assert.ok(offer_check.receiver.equals(receiver.publicKey));
    
  })

  it("Approves an offer", async () => {
    console.log("--------Approving offer...-------------");

    const offerId = "offer1";
    const[offerPDA, _] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("offer"), creator.publicKey.toBuffer(), Buffer.from(offerId)], program.programId);

    await program.methods.approveOfferCompletion()
    .accounts({
      creator: creator.publicKey,
      offer: offerPDA
    })
    .signers([creator])
    .rpc();

    
    const offer_check = await program.account.offer.fetch(offerPDA);
    assert.ok(offer_check.completed);    
  })


  
 it("Withdraws from offer", async () => {
    console.log("--------Withdrwawing from offer...---------");
    // await airdropSol(creator.publicKey, (1e9)* 1000); // 1000 sol

    const offerId = "offer1";
    const[offerPDA, _] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("offer"), creator.publicKey.toBuffer(), Buffer.from(offerId)], program.programId);
    
    await getBalances(offerPDA, receiver.publicKey, "Beginning");

    await program.methods.withdrawOffer()
    .accounts({
      receiver: receiver.publicKey,
      offer: offerPDA
    })
    .signers([receiver])
    .rpc();
    
    const offer_check = await program.account.offer.fetch(offerPDA);
    assert.ok(offer_check.withdrawn);


    await getBalances(offerPDA, receiver.publicKey, "After offer withdrawn");
  })


  // it("Withdraws from offer", async () => {
    
  // })

  
  async function getBalances(payerPubkey: anchor.web3.PublicKey, recipientPubkey: anchor.web3.PublicKey, timeframe: string) {
    const payerBalance = await provider.connection.getBalance(payerPubkey);
    const recipientBalance = await provider.connection.getBalance(recipientPubkey);
    console.log(`${timeframe} balances:`);
    console.log(`   Payer: ${payerBalance / LAMPORTS_PER_SOL}`);
    console.log(`   Recipient: ${recipientBalance / LAMPORTS_PER_SOL}`);
  }
});
