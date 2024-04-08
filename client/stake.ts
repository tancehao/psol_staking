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

let stsol_mint_pk = new Uint8Array([]);
let stsol_mint_kp = web3.Keypair.fromSecretKey(stsol_mint_pk);

let stsol_vault = await token.getOrCreateAssociatedTokenAccount(connection, wallet, stsol_mint_kp.publicKey, provider.wallet.publicKey, null, token.TOKEN_PROGRAM_ID, token.ASSOCIATED_TOKEN_PROGRAM_ID)
console.log("stsol_vault address: ", stsol_vault.address.toString());

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

// new synthetic account
let psol_vault = await token.getOrCreateAssociatedTokenAccount(connection, wallet, psol_pda, provider.wallet.publicKey, null, token.TOKEN_PROGRAM_ID, token.ASSOCIATED_TOKEN_PROGRAM_ID)

// helper fcn
async function print_state() {
    let balance = await connection.getTokenAccountBalance(stsol_vault.address)
    console.log("user's token stsol amount", balance.value.amount)

    balance = await connection.getTokenAccountBalance(psol_vault.address)
    console.log("user's token psol amount", balance.value.amount)

    balance = await connection.getTokenAccountBalance(vault_stsol_pda)
    console.log("total deposited token amount in pool", balance.value.amount)

    balance = await connection.getTokenAccountBalance(vault_slash_pool_pda)
    console.log("total slashed token amount in pool", balance.value.amount)
}


let operation_accounts = {
    stsol: stsol_mint_kp.publicKey,
    psol: psol_pda,
    vaultStsol: vault_stsol_pda,
    sender: provider.wallet.publicKey,
    senderStsol: stsol_vault.address,
    senderPsol: psol_vault.address,
    tokenProgram: token.TOKEN_PROGRAM_ID,
    clock: web3.SYSVAR_CLOCK_PUBKEY,
    reciept: reciept_pda,
}

console.log('');
console.log("state before staking:");
console.log("--------------------------------------");
await print_state()
console.log('');

// transfer X into program and get X synthetic tokens back
console.log('staking...')
await program.rpc.stake(new anchor.BN(100), {
    accounts: operation_accounts
});

console.log('');
console.log("state after staked:");
console.log("--------------------------------------");
await print_state()
console.log('');