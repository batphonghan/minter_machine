import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { MinterMachine } from '../target/types/minter_machine';

describe('minter_machine', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.MinterMachine as Program<MinterMachine>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
