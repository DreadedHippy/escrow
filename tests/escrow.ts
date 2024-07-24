import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { Escrow } from "../target/types/escrow";

const { SystemProgram } = anchor.web3;

describe("escrow", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  // let [walletAddr,] = await anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("offer")], programIdOfYourProgram);anchor


  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  let _offer = undefined;
  const [offerPDA, _] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("offer"), provider.wallet.publicKey.toBuffer()], program.programId);


  it("Is initialized!", async () => {
    // const holdingAccount = anchor.web3.Keypair.generate();
    // const offer_key = anchor.web3.Keypair.generate();
    const offerId = "offer 1";


    const amount = new anchor.BN(10);
    const tx = await program.methods
    .createOffer(amount, offerId, "Description one, this is a test offer", "", "")
    .accounts({
      offer: offerPDA,
      creator: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    })
    .rpc();

    
    const offer_check = await program.account.offer.fetch(offerPDA);

    assert.ok(offer_check.amount.eq(amount));
    
    console.log(`Your transaction signature ${tx}`);


    _offer = offerPDA;
  });

    const receiverAccount = anchor.web3.Keypair.generate();

  it("Is accepted", async () => {
    const offerPDA = _offer;

    const tx = await program.methods
      .acceptOffer()
      .accounts({
        offer: offerPDA,
        receiver: receiverAccount.publicKey,
      })
      .signers([receiverAccount])
      .rpc();


    // Fetch the newly updated account.

    const offer_check = await program.account.offer.fetch(offerPDA);

    // Assert that the offer is marked as accepted
    assert.ok(offer_check.accepted);

    // Assert that the offer is accepted by the signer/receiver
    assert.ok(offer_check.receiver.equals(receiverAccount.publicKey));

    _offer = offerPDA
  });

  describe("e2e escrow test", async () => {

      const offer = anchor.web3.Keypair.generate();
      const receiver = anchor.web3.Keypair.generate();
      // const creator = anchor.web3.Keypair.generate();
      // const amount = new anchor.BN(1000);
      // const [offerPDAF, _] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("offer"), provider.wallet.publicKey.toBuffer()], program.programId);



      // it("is created: full", async () => {
      //   const create_tx = await program.methods
      //     .createOffer(amount, "offeri", "Description for final test", "deliverableeesss", "category")
      //     .accounts({
      //       offer: offerPDAF,
      //       creator: provider.wallet.publicKey,
      //       systemProgram: SystemProgram.programId
      //     })
      //     .signers([offer])
      //     .rpc();

      //   const offer_check = await program.account.offer.fetch(offer.publicKey);
      //   assert.ok(offer_check.amount.eq(amount));

      //   console.log(`Your transaction signature for offer cretion: ${create_tx}`);

      //   // accept_tx
      // });

      // it("is accepted: full", async () => {
      //   const accept_tx = await program.methods
      //   .acceptOffer()
      //   .accounts({
      //     offer: offerPDAF,
      //     receiver: receiver.publicKey,
      //   })
      //   .signers([receiver])
      //   .rpc();

      //   console.log(`Your transaction signature for offer acceptance: ${accept_tx}`);
      //   const offer_accept_check = await program.account.offer.fetch(offer.publicKey);
      //   assert.ok(offer_accept_check.accepted);
      //   assert.ok(offer_accept_check.receiver.equals(receiver.publicKey));
      
      // });

      it("is accepted: full", async () => {
        const accept_tx = await program.methods
        .acceptOffer()
        .accounts({
            offer: offerPDA,
            receiver: receiver.publicKey
        })
        .signers([receiver])
        .rpc();

        console.log(`Your transaction signature for offer acceptence is ${accept_tx}`);
        const offer_accepted_check = await program.account.offer.fetch(offerPDA);
        assert.ok(offer_accepted_check.accepted);

    
      });

      // let offerPDA = _offer;

      it("is approved: full", async () => {
        const approve_tx = await program.methods
        .approvePayment()
        .accounts({
          offer: offerPDA,
          creator: provider.wallet.publicKey,
          // systemProgram: SystemProgram.programId
        })
        .rpc();

        console.log(`Your transaction signature for offer payment approval ${approve_tx}`);
        const offer_approve_check = await program.account.offer.fetch(offerPDA);
        assert.ok(offer_approve_check.completed);
        console.log(`Offer receiver ${offer_approve_check.receiver}`);
      });

      
      it("is paid: full", async () => {
        const receive_payment_tx = await program.methods
          .receivePayment()
          .accounts({
            offer: offerPDA,
            receiver: receiverAccount.publicKey,
            systemProgram: SystemProgram.programId
          })
          .signers([receiverAccount])
          .rpc();

        console.log(`Your transaction signature for payment receival: ${receive_payment_tx} `);
        const offer_receive_payment_check = await program.account.offer.fetch(offer.publicKey);
        assert.ok(offer_receive_payment_check.paymentReceived)

      
      })


    })

    

    
    
  // });

  // it("is retrieved", async () => {

  //   const holdingAccount = anchor.web3.Keypair.generate();
  //   const offer = anchor.web3.Keypair.generate();

  //   const amount = new anchor.BN(10);
  //   const tx = await program.methods
  //   .createOffer(amount, "Description one, this is a test offer", "deliver", "cat")
  //   .accounts({
  //     offer: offer.publicKey,
  //     holdingAccount: holdingAccount.publicKey,
  //     creator: provider.wallet.publicKey,
  //     systemProgram: SystemProgram.programId
  //   })
  //   .signers([offer, holdingAccount])
  //   .rpc();
  //   const offer_check = await program.account.offer.fetch(offer.publicKey);

  //   assert.ok(offer_check.amount.eq(amount));

  //   const tx2 = await program.methods
  //   .getOffer()
  //   .accounts({
  //     offer: offer.publicKey
  //   }).rpc();

  //   console.log(`${tx2}`);    
  // })



  

  // it("Is approved", async () => {
  //   const offer = anchor.web3.Keypair.generate();
  //   const holdingAccount = anchor.web3.Keypair.generate();
  //   const receiverAccount = anchor.web3.Keypair.generate();
  //   const creatorKeypair = anchor.web3.Keypair.generate();
  //   const amount = new anchor.BN(100);

  //   await provider.connection.confirmTransaction(
  //     await provider.connection.requestAirdrop(creatorKeypair.publicKey, 10_000_000_000)
  //   )

  //   await provider.connection.confirmTransaction(
  //     await provider.connection.requestAirdrop(holdingAccount.publicKey, 10_000_000_000)
  //   )

  //   await provider.connection.confirmTransaction(
  //     await provider.connection.requestAirdrop(receiverAccount.publicKey, 10_000_000_000)
  //   );

  //   const tx = await program.methods
  //   .createOffer(amount, "Description one, this is a test offer", "deliver", "cat")
  //   .accounts({
  //     offer: offer.publicKey,
  //     holdingAccount: holdingAccount.publicKey,
  //     creator: provider.wallet.publicKey,
  //     systemProgram: SystemProgram.programId
  //   })
  //   .signers([offer, holdingAccount])
  //   .rpc();
  //   const offer_check = await program.account.offer.fetch(offer.publicKey);

  //   assert.ok(offer_check.amount.eq(amount));

  //   console.log(`Your transaction signature ${tx}`);

  //   const tx2 = await program.methods
  //   .acceptOffer()
  //   .accounts({
  //     offer: offer.publicKey,
  //     receiver: provider.wallet.publicKey
  //   })
  //   // .signers([receiverAccount])
  //   .rpc();

  //   const offer_check_accept = await program.account.offer.fetch(offer.publicKey);
  //   assert.ok(offer_check_accept.accepted);


  //   const tx3 = await program.methods
  //   .approvePayment()
  //   .accounts({
  //     offer: offer.publicKey,
  //     creator: provider.wallet.publicKey,
  //     receiver: provider.wallet.publicKey,
  //     holdingAccount: holdingAccount.publicKey,
  //     systemProgram: SystemProgram.programId
  //   })
  //   // .signers([holdingAccount])
  //   .rpc();

  //  _offer = offer;    
    

  // console.log("Done before accept");

  //  const tx_accept = await program.methods
  //    .acceptOffer()
  //    .accounts({
  //      offer: offer.publicKey,
  //      receiver: receiverAccount.publicKey,
  //    })
  //    .signers([receiverAccount])
  //  .rpc();


  // / // Fetch the newly updated account.
  // / const offer_check_accept = await program.account.offer.fetch(offer.publicKey);

  // / // Assert that the offer is marked as accepted
  // / assert.ok(offer_check_accept.accepted);

  // / // Assert that the offer is accepted by the signer/receiver
  // / assert.ok(offer_check_accept.receiver.equals(receiverAccount.publicKey));

  // / console.log(`offer pc ${offer.publicKey}`);
  // / console.log(`holding pc: ${holdingAccount.publicKey}`),
  // / console.log(`receiver pc: ${receiverAccount.publicKey}`)
  // / const tx_approve = await program.methods
  // /   .approvePayment()
  // /   .accounts({
  // /     offer: offer.publicKey,
  // /     holdingAccount: holdingAccount.publicKey,
  // /     creator: provider.wallet.publicKey,
  // /     receiver: receiverAccount.publicKey,
  // /     systemProgram: SystemProgram.programId
  // /   })
  // /   .signers([holdingAccount])
  // /   .rpc();

  // / // const offer_check_approve = await program.account.offer.fetch(offer.publicKey);

    
  // / // assert.ok(offer_check_approve.completed);
  // })
});
