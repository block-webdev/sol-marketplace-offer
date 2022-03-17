import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { NmOffer } from '../target/types/nm_offer';

describe('nm-offer', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.NmOffer as Program<NmOffer>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
