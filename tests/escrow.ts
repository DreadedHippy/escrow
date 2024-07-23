import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { Escrow } from "../target/types/escrow";

const { SystemProgram } = anchor.web3;

describe("escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  let _offer = undefined;

  it("Is initialized!", async () => {
    const holdingAccount = anchor.web3.Keypair.generate();
    const offer = anchor.web3.Keypair.generate();


    const amount = new anchor.BN(10);
    const tx = await program.methods
    .createOffer(amount, "Description one, this is a test offer", "deliver", "cat")
    .accounts({
      offer: offer.publicKey,
      holdingAccount: holdingAccount.publicKey,
      creator: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    })
    .signers([offer, holdingAccount])
    .rpc();
    const offer_check = await program.account.offer.fetch(offer.publicKey);

    assert.ok(offer_check.amount.eq(amount));
    
    console.log(`Your transaction signature ${tx}`);

    _offer = offer;    
  });

  it("Is accepted", async () => {
    const offer = _offer;
    const receiverAccount = anchor.web3.Keypair.generate();

    const tx = await program.methods
      .acceptOffer()
      .accounts({
        offer: offer.publicKey,
        receiver: receiverAccount,
      })
      .signers([receiverAccount])
      .rpc();


    // Fetch the newly updated account.
    const offer_check = await program.account.offer.fetch(offer.publicKey);

    // Assert that the offer is marked as accepted
    assert.ok(offer_check.accepted);

    // Assert that the offer is accepted by the signer/receiver
    assert.ok(offer_check.receiver.equals(receiverAccount.publicKey));

    _offer = offer
  })

  it("Is approved", async () => {
    const offer = anchor.web3.Keypair.generate();
    const holdingAccount = anchor.web3.Keypair.generate();
    const receiverAccount = anchor.web3.Keypair.generate();
    const creatorKeypair = anchor.web3.Keypair.generate();
    const amount = new anchor.BN(100);

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(creatorKeypair.publicKey, 10_000_000_000)
    )

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(holdingAccount.publicKey, 10_000_000_000)
    )

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(receiverAccount.publicKey, 10_000_000_000)
    );

    const tx = await program.methods
    .createOffer(amount, "Description one, this is a test offer", "deliver", "cat")
    .accounts({
      offer: offer.publicKey,
      holdingAccount: holdingAccount.publicKey,
      creator: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    })
    .signers([offer, holdingAccount])
    .rpc();
    const offer_check = await program.account.offer.fetch(offer.publicKey);

    assert.ok(offer_check.amount.eq(amount));

    console.log(`Your transaction signature ${tx}`);

    const tx2 = await program.methods
    .acceptOffer()
    .accounts({
      offer: offer.publicKey,
      receiver: provider.wallet.publicKey
    })
    // .signers([receiverAccount])
    .rpc();

    const offer_check_accept = await program.account.offer.fetch(offer.publicKey);
    assert.ok(offer_check_accept.accepted);


    const tx3 = await program.methods
    .approvePayment()
    .accounts({
      offer: offer.publicKey,
      creator: provider.wallet.publicKey,
      receiver: provider.wallet.publicKey,
      holdingAccount: holdingAccount.publicKey,
      systemProgram: SystemProgram.programId
    })
    // .signers([holdingAccount])
    .rpc();

    _offer = offer;    
    

    // console.log("Done before accept");

    // const tx_accept = await program.methods
    //   .acceptOffer()
    //   .accounts({
    //     offer: offer.publicKey,
    //     receiver: receiverAccount.publicKey,
    //   })
    //   .signers([receiverAccount])
    //   .rpc();


    // // Fetch the newly updated account.
    // const offer_check_accept = await program.account.offer.fetch(offer.publicKey);

    // // Assert that the offer is marked as accepted
    // assert.ok(offer_check_accept.accepted);

    // // Assert that the offer is accepted by the signer/receiver
    // assert.ok(offer_check_accept.receiver.equals(receiverAccount.publicKey));

    // console.log(`offer pc ${offer.publicKey}`);
    // console.log(`holding pc: ${holdingAccount.publicKey}`),
    // console.log(`receiver pc: ${receiverAccount.publicKey}`)
    // const tx_approve = await program.methods
    //   .approvePayment()
    //   .accounts({
    //     offer: offer.publicKey,
    //     holdingAccount: holdingAccount.publicKey,
    //     creator: provider.wallet.publicKey,
    //     receiver: receiverAccount.publicKey,
    //     systemProgram: SystemProgram.programId
    //   })
    //   .signers([holdingAccount])
    //   .rpc();

    // // const offer_check_approve = await program.account.offer.fetch(offer.publicKey);

    
    // // assert.ok(offer_check_approve.completed);
  })
});
