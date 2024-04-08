import * as web3 from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
import * as token from '@solana/spl-token';

import { Program } from '@project-serum/anchor';
import { AnchorStake } from '../target/types/anchor_stake';

// Configure the client to use the local cluster.
let provider = anchor.Provider.env();

let connection = provider.connection;
anchor.setProvider(provider);

const program = anchor.workspace.AnchorStake as Program<AnchorStake>;

let wallet = pg.wallet.keypair

// create a new mint to send to program
let stsol_mint_kp = web3.Keypair.generate();
console.log("Generated keypair to create the stsol mint! publickey: ", stsol_mint_kp.publicKey.toString());
console.log("privatekey: ", stsol_mint_kp.secretKey.toString());
await token.createMint(
    connection,
    wallet,
    stsol_mint_kp.publicKey,
    null,
    18,
    stsol_mint_kp, undefined, token.TOKEN_PROGRAM_ID
)
console.log("created stsol mint");

let stsol_vault = await token.getOrCreateAssociatedTokenAccount(connection, wallet, stsol_mint_kp.publicKey, provider.wallet.publicKey, null, token.TOKEN_PROGRAM_ID, token.ASSOCIATED_TOKEN_PROGRAM_ID)
console.log("stsol_vault address: ", stsol_vault.address.toString());

let init_x = 100
await token.mintTo(connection, wallet, stsol_mint_kp.publicKey, stsol_vault.address, stsol_mint_kp, init_x, [], null, token.TOKEN_PROGRAM_ID)

const [psol_pda, sb] =
    await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("psol"), stsol_mint_kp.publicKey.toBuffer()],
        program.programId
    );

const [vault_stsol_pda, vb] =
    await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("vault_stsol"), stsol_mint_kp.publicKey.toBuffer()],
        program.programId
    );

const [vault_slash_pool_pda, vs] =
    await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("vault_slash_pool"), stsol_mint_kp.publicKey.toBuffer()],
        program.programId
    );

//b"reciept", token_x.key().as_ref(), sender.key().as_ref()
const [reciept_pda, rb] =
    await anchor.web3.PublicKey.findProgramAddress(
        [
            Buffer.from("reciept"),
            stsol_mint_kp.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer(),
        ],
        program.programId
    );

// let balance = await connection.getTokenAccountBalance(stsol_vault.address)
// console.log('MY token amount', balance.value.amount)
await program.rpc.initialize({
    accounts: {
        stsol: stsol_mint_kp.publicKey,
        psol: psol_pda,
        vaultStsol: vault_stsol_pda,
        vaultSlashPool: vault_slash_pool_pda,
        payer: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: token.ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY
    },
})
// create a new staker account for tokenX
await program.rpc.newStaker({ accounts: {
    stsol: stsol_mint_kp.publicKey,
    reciept: reciept_pda,
    sender: provider.wallet.publicKey,
    systemProgram: web3.SystemProgram.programId,
}});
console.log('User joined the stake pool');
