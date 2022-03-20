import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { NmOffer } from '../target/types/nm_offer';
import { PublicKey, SystemProgram, Transaction, Connection, Commitment } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";

describe('nm-offer', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();// new anchor.Provider(connection, wallet, options);
  anchor.setProvider(provider);

  const program = anchor.workspace.NmOffer as Program<NmOffer>;

  let mintA = null as Token;
  let mintB = null as Token;
  let initializerTokenAccountA = null;
  let initializerTokenAccountB = null;
  let takerTokenAccountA = null;
  let takerTokenAccountB = null;
  let vault_account_pda = null;
  let vault_account_bump = null;
  let vault_authority_pda = null;

  const takerAmount = 1000;
  const initializerAmount = 500;

  const escrowAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const takerMainAccount = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 1000000000),
      "processed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: initializerMainAccount.publicKey,
            lamports: 100000000,
          }),
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: takerMainAccount.publicKey,
            lamports: 100000000,
          })
        );
        return tx;
      })(),
      [payer]
    );

    mintA = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    mintB = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    initializerTokenAccountA = await mintA.createAccount(initializerMainAccount.publicKey);
    takerTokenAccountA = await mintA.createAccount(takerMainAccount.publicKey);

    initializerTokenAccountB = await mintB.createAccount(initializerMainAccount.publicKey);
    takerTokenAccountB = await mintB.createAccount(takerMainAccount.publicKey);

    await mintA.mintTo(
      initializerTokenAccountA,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await mintB.mintTo(
      takerTokenAccountB,
      mintAuthority.publicKey,
      [mintAuthority],
      takerAmount
    );

    let _initializerTokenAccountA = await mintA.getAccountInfo(initializerTokenAccountA);
    let _takerTokenAccountB = await mintB.getAccountInfo(takerTokenAccountB);

    assert.ok(_initializerTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_takerTokenAccountB.amount.toNumber() == takerAmount);
  });


  it('Is initialized!', async () => {
    // Add your test here.
    let rand = anchor.web3.Keypair.generate().publicKey;

    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
      [rand.toBuffer()],
      program.programId
    );
    vault_account_pda = _vault_account_pda;
    vault_account_bump = _vault_account_bump;

    var myToken = new Token(
      provider.connection,
      mintA.publicKey,
      TOKEN_PROGRAM_ID,
      initializerMainAccount
    );
    var destAccount = await myToken.getOrCreateAssociatedAccountInfo(program.programId);
    console.log('1111111111', destAccount);


    let tx = await program.rpc.initialize(
      vault_account_bump,
      {
        accounts: {
          owner: initializerMainAccount.publicKey,
          pool: destAccount.address,
          rand: rand,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [initializerMainAccount]
      }
    );

    console.log("Your transaction signature", tx);

    // await program.rpc.initOfferData(
    //   new anchor.BN(1),
    //   new anchor.BN(2),
    //   new anchor.BN(3),
    //   {
    //     accounts: {

    //     }

    //   }
    // );
  });
});
