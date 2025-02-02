import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Token } from "../target/types/token";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

describe("nft-token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Token as Program<Token>;
  
  let mint: anchor.web3.Keypair;
  let ownerTokenAccount: anchor.web3.PublicKey;
  let recipientTokenAccount: anchor.web3.PublicKey;
  
  const recipient = anchor.web3.Keypair.generate();

  before(async () => {
    mint = anchor.web3.Keypair.generate();
    
    // Create associated token accounts
    ownerTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      provider.wallet.publicKey
    );
    
    recipientTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      recipient.publicKey
    );
  });

  it("Initializes NFT", async () => {
    await program.methods
      .initializeNft(
        "My NFT",
        "MNFT",
        "https://example.com/nft.json"
      )
      .accounts({
        payer: provider.wallet.publicKey,
        mint: mint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([mint])
      .rpc();
  });

  it("Mints NFT", async () => {
    // Create token account if it doesn't exist
    await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint.publicKey,
      provider.wallet.publicKey
    );

    await program.methods
      .mintNft()
      .accounts({
        mint: mint.publicKey,
        token: ownerTokenAccount,
        owner: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Verify mint
    const balance = await provider.connection.getTokenAccountBalance(ownerTokenAccount);
    assert.equal(balance.value.uiAmount, 1);
  });

  it("Transfers NFT", async () => {
    // Create recipient token account
    await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint.publicKey,
      recipient.publicKey
    );

    await program.methods
      .transferNft()
      .accounts({
        from: ownerTokenAccount,
        to: recipientTokenAccount,
        owner: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Verify transfer
    const recipientBalance = await provider.connection.getTokenAccountBalance(recipientTokenAccount);
    assert.equal(recipientBalance.value.uiAmount, 1);
    
    const previousOwnerBalance = await provider.connection.getTokenAccountBalance(ownerTokenAccount);
    assert.equal(previousOwnerBalance.value.uiAmount, 0);
  });
});
