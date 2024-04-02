import React, { useState, useEffect } from 'react'
import { WalletModalButton } from '@solana/wallet-adapter-react-ui'
import { PublicKey, Connection, SystemProgram } from '@solana/web3.js'
import { Program, AnchorProvider, BN} from '@coral-xyz/anchor'
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync} from '@solana/spl-token'
import idl from './../../target/idl/staking_thing.json'
import { useWallet } from '@solana/wallet-adapter-react'
import { Buffer } from 'buffer'
import './styles.css'
import  { sendTransaction2, formatPublicKey, parseGameState, parseWagerType }from './utils.ts'

const programID = new PublicKey(idl.metadata.address)
const network = "https://worried-chiquia-fast-devnet.helius-rpc.com/"
const opts = { preflightCommitment: "processed" }

const App = () => {
  const wallet = useWallet()
  const [vault, setVaults] = useState([])
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
  
      // Fetch all game accounts
      const vaults = await program.account.vault.all();
      
      // Fetch and set the current blockchain timestamp for 
      
      const slot = await connection.getSlot();
      const currentTimestamp = await connection.getBlockTime(slot);
      if (!currentTimestamp) throw new Error('Failed to get current blockchain time');
      setCurrentBlockchainTime(currentTimestamp);
  
      setVaults(vaults);

      console.log("Vaults:", vaults);
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

      const txId = await sendTransaction2(provider, instruction, undefined, 5000);

      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error creating game:", error);
    }
  };

  const deposit = async (vault, amount) => {
    try {
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
  
      // Here gamePubKey is the PublicKey of the game you're joining
      // creatorAddressPubKey is the PublicKey of the creator for the game

      const playerPubKey = provider.wallet.publicKey;

      // Generate the gamba_state PDA 
      const [gambaState, gameAccountBump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("GAMBA_STATE"),
        ],
        program.programId
      );
  
      // Find the associated token accounts for the player and the creator
      const playerAta = getAssociatedTokenAddressSync(
        game.account.mint,
        playerPubKey,
      );
  
      const creatorAta = getAssociatedTokenAddressSync(
        game.account.mint,
        creatorAddressPubKey,
      );

      const [gameAccountTaAccountPDA, gameAccountTaAccountBump] = PublicKey.findProgramAddressSync(
        [game.publicKey.toBuffer()],
        program.programId
      );

      const gameAccountPubKey = (game.publicKey instanceof PublicKey) ? game.publicKey : new PublicKey(game.publicKey);
  
      const instruction = await program.methods.joinGame(new BN(creatorFee), new BN(wager))
        .accounts({
          gameAccount: gameAccountPubKey,
          gambaState: gambaState,
          gameAccountTa: gameAccountTaAccountPDA,
          mint: game.account.mint,
          playerAccount: playerPubKey,
          playerAta: playerAta,
          creatorAddress: creatorAddressPubKey,
          creatorAta: creatorAta,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
        }).instruction();
  
      const txId = await sendTransaction2(provider, instruction);
      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error joining game:", error);
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
      {/* <div>
      {games.map((game, index) => {
        const expirationTimestamp = new BN(game.account.gameExpirationTimestamp).toNumber();
        const timeUntilExpiration = currentBlockchainTime ? Math.max(0, expirationTimestamp - currentBlockchainTime) : null;
  
        return (
          <div key={index} className="gameCard">
            <div>Game Account Public Key: {formatPublicKey(game.publicKey)}</div>
            <div>Game Maker: {formatPublicKey(game.account.gameMaker)}</div>
            <div>State: {parseGameState(game.account.state)}</div>
            <div>Mint: {formatPublicKey(game.account.mint)}</div>
            <div>Max Players: {game.account.maxPlayers.toString()}</div>
            <div>Winners: {game.account.winners.toString()}</div>
            <div>Game ID: {game.account.gameId.toString()}</div>
            <div>Game Expiration Timestamp: {game.account.gameExpirationTimestamp.toString()}</div>
            <div>Time Until Expiration: {timeUntilExpiration !== null ? `${timeUntilExpiration} seconds` : 'Loading...'}</div>
            <div>Unique Identifier: {game.account.uniqueIdentifier.toString()}</div>
            <div>Wager Type: {parseWagerType(game.account.wagerType)}</div>
            <div>Wager: {game.account.wager.toString()}</div>
            <div>
                Players:
                {game.account.players.map((player, playerIndex) => (
                  <div key={playerIndex} className="playerInfo">
                    <div>Creator ATA: {formatPublicKey(player.creatorAddressAta)}</div>
                    <div>User ATA: {formatPublicKey(player.userAta)}</div>
                    <div>Creator Fee: {player.creatorFeeAmount.toString()}</div>
                    <div>Gamba Fee: {player.gambaFeeAmount.toString()}</div>
                    <div>Wager: {player.wager.toString()}</div>
                  </div>
                ))}
              </div>
            <div className="buttonContainer">
              <div className="joinLeaveButtons">
                <button onClick={() => joinGame(game, new PublicKey("5r5Sos7CQUNdN9EpwwSu1ujGVnsChv24TmrtjTWkAdNj"), 100, 5000000)}>Join Game</button>
                <button onClick={() => leaveGame(game)}>Leave Game</button>
              </div>
              {(parseGameState(game.account.state) === 'Playing' || timeUntilExpiration === 0) && (
                <button className="settleButton" onClick={() => settleGame(game)}>Settle Game</button>
               )}
            </div>
          </div>
        );
      })}
      </div> */}
    </div>
  );
}

export default App;