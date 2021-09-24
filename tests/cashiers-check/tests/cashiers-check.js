const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const assert = require("assert");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");



describe("cashiers-check", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.CashiersCheck;

  let mint = null;
  let god = null;
  let receiver = null;

  it("Sets up initial test state", async () => {
    const [_mint, _god] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(1000000)
    );
    //return value for the mint 
    //
    mint = _mint;
    //return vlaue for the vault
    god = _god;
    //both are owned by the token program
    //so i think what's happening is it just creates a mint and then mints a bunch of tokens to the vault
    //mint authority retained by the provider's wallet

    //call to create a token account specific to the mint we just created
    //so again this transfer will be with a specific tokem, not sol
    receiver = await serumCmn.createTokenAccount(
      program.provider,
      mint,
      program.provider.wallet.publicKey
    );
    //receiver is just another token  account that we can send things to?
  });

  const check = anchor.web3.Keypair.generate();
  const vault = anchor.web3.Keypair.generate();

  let checkSigner = null;

  it("Creates a check!", async () => {

    //checksigner is the first PDA. or rather it's not a pda yet, just an address we can use to make a pda
    let [_checkSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check.publicKey.toBuffer()],
      program.programId
    );
    checkSigner = _checkSigner;

    await program.rpc.createCheck(new anchor.BN(100), "Hello world", nonce, {
      accounts: {
        check: check.publicKey,
        vault: vault.publicKey,
        checkSigner,
        from: god,
        to: receiver,
        owner: program.provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [check, vault],
      instructions: [
        await program.account.check.createInstruction(check, 300),
        ...(await serumCmn.createTokenAccountInstrs(
          program.provider,
          vault.publicKey,
          mint,
          checkSigner
        )),
      ],
    });


    /*


    in this set up, you have the vault token account which just stores the tokens,
    and you have the check signer which owns the vault account and can move shit out of it

    but in my case, i want to send funds directly to the pda (the check signer)
    so let me just make sure i can do that

    okay so now i need to figure out how to sign a message with a program

    so if i can put it in sum
    1. create a keypair for the vault
    2. find a PDA address for the given program that will own the vault (called checksigner)
    3. create a vault tokenAccount that is owned by the checkSigner (so that the checksigner can sign txs to move money out of the vault)
    4. move money out of the vault from the program.

    -got it

    got itmess

    //so the tokenAccount always has the owner (sometimes called programID) as the TOKEN_PROGRAM_ID
    //i think this means it's the main TOKEN PROGRAM that can transfer it and shit
    //when you intitialiZe the account, there's another owner field. which is different i guess?
    //same as initialize on my example from the medium post. not sure about this

    //so there's some mismatch distinction between owner on token account and regular account

    okay wait i see it now. the owner is set in the initialize account
    i am still a little confused with the owner as programId and the owner as controller 
    i think they are sort of using it as both


    okay so there's the concept of owner in the createAccount. Sometimes passed in as programID. 
    the anchor_lang docs clearly state owner as "program that owns this account" in the AccountInfo struct
    https://docs.rs/anchor-lang/0.16.1/anchor_lang/prelude/struct.AccountInfo.html

    https://docs.solana.com/developing/programming-model/accounts

    so i guess you just have to initialize it with the system program but then you can change it to whatever you want

    based on this code, PDAs (and i'm assuming other accounts) can definitely own accounts
    and it's all the same owner field.

    ---- here is the answer ----
    It would be much easier for Alice if she just had one private key for all her token accounts and this is exactly how 
    the token program does it! It assigns each token account an owner. Note that this token account owner attribute is 
    not the same as the account owner. The account owner is an internal Solana attribute that will always be a program. 
    The new token owner attribute is something the token program declares in user space (i.e. in the program they are building). 
    It's encoded inside a token account's data, in addition to other properties (opens new window)such as the balance of tokens the account holds. 
    What this also means is that once a token account has been set up, its private key is useless, only 
    its token owner attribute matters. And the token owner attribute is going to be some other address, in our case 
    Alice's and Bob's main account respectively. When making a token transfer they simply have to sign the tx (tx=transaction) 
    with the private key of their main account.

    https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/



    //okay so you have the account owner. and you 
    
    //so the account owner is the system program
    //the token account owner is the account that can sign for transactions
    on behalf of the token account

    the token account's owner can only be set once.
    
    so what does that have to do with a pda??

    im guessing pda's can only be singed by the program??


    */


    //we use this function to actually create the vault account and assign the check_signer as owner
    //https://github.com/project-serum/serum-ts/blob/master/packages/common/src/index.ts#L139
    /*
    export async function createTokenAccountInstrs(
      provider: Provider,
      newAccountPubkey: PublicKey,
      mint: PublicKey,
      owner: PublicKey,
      lamports?: number,
    ): Promise<TransactionInstruction[]> 
    */

    const checkAccount = await program.account.check.fetch(check.publicKey);
    assert.ok(checkAccount.from.equals(god));
    assert.ok(checkAccount.to.equals(receiver));
    assert.ok(checkAccount.amount.eq(new anchor.BN(100)));
    assert.ok(checkAccount.memo === "Hello world");
    assert.ok(checkAccount.vault.equals(vault.publicKey));
    assert.ok(checkAccount.nonce === nonce);
    assert.ok(checkAccount.burned === false);

    let vaultAccount = await serumCmn.getTokenAccount(
      program.provider,
      checkAccount.vault
    );
    assert.ok(vaultAccount.amount.eq(new anchor.BN(100)));
  });

  it("Cashes a check", async () => {
    await program.rpc.cashCheck({
      accounts: {
        check: check.publicKey,
        vault: vault.publicKey,
        checkSigner: checkSigner,
        to: receiver,
        owner: program.provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const checkAccount = await program.account.check.fetch(check.publicKey);
    assert.ok(checkAccount.burned === true);

    let vaultAccount = await serumCmn.getTokenAccount(
      program.provider,
      checkAccount.vault
    );
    assert.ok(vaultAccount.amount.eq(new anchor.BN(0)));

    let receiverAccount = await serumCmn.getTokenAccount(
      program.provider,
      receiver
    );
    assert.ok(receiverAccount.amount.eq(new anchor.BN(100)));
  });
});
