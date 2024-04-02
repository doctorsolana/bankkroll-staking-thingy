import React, { useState, useEffect } from 'react'
import { WalletModalButton } from '@solana/wallet-adapter-react-ui'
import { PublicKey, Connection, SystemProgram } from '@solana/web3.js'
import { Program, AnchorProvider, BN} from '@coral-xyz/anchor'
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync} from '@solana/spl-token'
import idl from './../../target/idl/staking_thing.json'
import { useWallet } from '@solana/wallet-adapter-react'
import { Buffer } from 'buffer'
import './styles.css'
import  { sendTransaction, formatPublicKey }from './utils.ts'

const programID = new PublicKey(idl.metadata.address)
const network = "https://worried-chiquia-fast-devnet.helius-rpc.com/"
const opts = { preflightCommitment: "processed" }

const App = () => {
  const wallet = useWallet()
  const [vaults, setVaults] = useState([])
  const [currentBlockchainTime, setCurrentBlockchainTime] = useState(null)
  const [mintAddress, setMintAddress] = useState('')

  // fetch vaults
  useEffect(() => {
    const fetchInterval = 2500; // Fetch every n milliseconds 
  
    const intervalId = setInterval(() => {
      if (wallet.connected) {
        fetchVaults();
      }
    }, fetchInterval);
  
    return () => clearInterval(intervalId);
  }, [wallet.connected]); // Re-run effect if wallet.connected changes

  const getProvider = () => {
    const connection = new Connection(network, opts.preflightCommitment);
    const provider = new AnchorProvider(connection, wallet, opts.preflightCommitment);
    return provider;
  };

  const fetchVaults = async () => {
    try {
      const connection = new Connection(network, opts.preflightCommitment);
      const provider = new AnchorProvider(connection, wallet, opts.preflightCommitment);
      const program = new Program(idl, programID, provider);
  
      // Fetch all vaults
      const vaultsInput = await program.account.vault.all();
      
      // Fetch and set the current blockchain timestamp for the UI
      const slot = await connection.getSlot();
      const currentTimestamp = await connection.getBlockTime(slot);
      if (!currentTimestamp) throw new Error('Failed to get current blockchain time');
      setCurrentBlockchainTime(currentTimestamp);
      
      console.log(vaultsInput)
      setVaults(vaultsInput);
    } catch (error) {
      console.error("Error fetching vaults or blockchain timestamp:", error);
    }
  };

  const createVault = async () => {
    try {
      const provider = getProvider()
      const program = new Program(idl, programID, provider)

      const mint = new PublicKey(mintAddress) // Mint of the token you want to use for the vault

      // Correcting the seeds for vault account initialization
      const [vault, vaultBump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("VAULT"), 
          mint.toBuffer(), 
        ],
        program.programId
      );

      // Seeds for vault_ta account initialization
      const [vault_ta, vaultTaBump] = PublicKey.findProgramAddressSync(
        [
          vault.toBuffer(), 
        ],
        program.programId
      );

      // Prepare instruction using the program method
      const instruction = await program.methods.createVault()
      .accounts({
        signer: provider.wallet.publicKey,
        mint: mint,
        vault: vault,
        vaultTa: vault_ta,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      }).instruction();

      const txId = await sendTransaction(provider, instruction, undefined, 5000);

      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error creating game:", error);
    }
  };

  const deposit = async (selectedVault, depositAmount) => {
    try {
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
      
      // Find the PDA for the user's account within this vault
      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [provider.wallet.publicKey.toBuffer(), selectedVault.publicKey.toBuffer()],
        program.programId
      );
  
      // Find the PDA for the vault account
      const [vaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("VAULT"), selectedVault.account.mint.toBuffer()],
        program.programId
      );
  
      // Find the PDA for the vault's token account
      const [vaultTaPDA] = PublicKey.findProgramAddressSync(
        [selectedVault.publicKey.toBuffer()],
        program.programId
      );
  
      // The user's associated token account for the mint
      const signerAta = getAssociatedTokenAddressSync(
        selectedVault.account.mint,
        provider.wallet.publicKey,
      );

      // Construct the deposit instruction
      const instruction = await program.methods.deposit(new BN(depositAmount))
        .accounts({
          signer: provider.wallet.publicKey,
          userAccount: userAccountPDA,
          mint: selectedVault.account.mint,
          vault: vaultPDA,
          signerAta: signerAta,
          vaultTa: vaultTaPDA,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }).instruction();
  
      // Send the transaction
      const txId = await sendTransaction(provider, instruction, undefined, 5000);
      console.log("Deposit Transaction ID:", txId);
    } catch (error) {
      console.error("Error making deposit:", error);
    }
  };
  

  
  return (
    <div>
      <WalletModalButton />
      <div>Wallet Public Key: {wallet.publicKey?.toString()}</div>
      <div>
        <input
          type="text"
          placeholder="Enter Mint Address"
          value={mintAddress}
          onChange={(e) => setMintAddress(e.target.value)}
        />
        <button onClick={createVault}>Create Vault</button>
      </div>
      <div className="button-row">
        <button onClick={fetchVaults}>Refresh Vaults</button>
      </div>
      <div>
      {vaults.map((vault, index) => {
        return (
          // <div> hello </div>
          <div key={index} className="gameCard">
            <div>Vault Public Key: {formatPublicKey(vault.publicKey)}</div>
            <div>Mint: {formatPublicKey(vault.account.mint)}</div>
            <div>Token account: {formatPublicKey(vault.account.tokenAccount)}</div>
            <div>Total LP outstanding: {vault.account.totalLp.toString()}</div>
            <div className="buttonContainer">
              <div className="joinLeaveButtons">
                <button onClick={() => deposit(vault, 100_000)}>Deposit</button>
              </div>
            </div>
          </div>
        );
      })}
      </div>
    </div>
  );
}

export default App;