import React, { useState, useEffect } from 'react';
import { WalletModalButton } from '@solana/wallet-adapter-react-ui';
import { PublicKey, Connection, SystemProgram, TransactionMessage, VersionedTransaction } from '@solana/web3.js';
import { Program, AnchorProvider, BN} from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync} from '@solana/spl-token';
import idl from './idl.json';
import { useWallet } from '@solana/wallet-adapter-react';
import { Buffer } from 'buffer';
import './styles.css'

const programID = new PublicKey(idl.metadata.address);
const network = "https://worried-chiquia-fast-devnet.helius-rpc.com/";
const opts = { preflightCommitment: "processed" };

const WagerType = {
  SameWager: 0, 
  CustomWager: 1, 
};

// Utility functions
const formatPublicKey = (publicKey) => publicKey.toString();
const formatBN = (bn) => bn.toString();
const parseGameState = (stateObject) => {
  const stateKeys = Object.keys(stateObject);
  if (stateKeys.length > 0) {
    const currentState = stateKeys[0];
    return currentState.charAt(0).toUpperCase() + currentState.slice(1);
  }
  return "Unknown State";
};
const parseWagerType = (wagerTypeObject) => {
  const wagerTypeKeys = Object.keys(wagerTypeObject);
  return wagerTypeKeys.length > 0 ? wagerTypeKeys[0] : "Unknown";
};


const App = () => {
  const wallet = useWallet();
  const [games, setGames] = useState([]);
  const [maxPlayers, setMaxPlayers] = useState(2); // Default to 2 players
  const [tokenMint, setTokenMint] = useState(''); 
  const [currentBlockchainTime, setCurrentBlockchainTime] = useState(null);


  useEffect(() => {
    if (wallet.connected) {
      fetchGames();
    }
  }, [wallet.connected]);

  const getProvider = () => {
    const connection = new Connection(network, opts.preflightCommitment);
    const provider = new AnchorProvider(connection, wallet, opts.preflightCommitment);
    return provider;
  };

  const fetchGames = async () => {
    try {
      const connection = new Connection(network, opts.preflightCommitment);
      const provider = new AnchorProvider(connection, wallet, opts.preflightCommitment);
      const program = new Program(idl, programID, provider);
  
      // Fetch all game accounts
      const gameAccounts = await program.account.game.all();
      
      // Fetch and set the current blockchain timestamp for displaying how long til games expire
      const slot = await connection.getSlot();
      const currentTimestamp = await connection.getBlockTime(slot);
      if (!currentTimestamp) throw new Error('Failed to get current blockchain time');
      setCurrentBlockchainTime(currentTimestamp);
  
      setGames(gameAccounts);
    } catch (error) {
      console.error("Error fetching game accounts or blockchain timestamp:", error);
    }
  };
  

  const createGame = async () => {
    try {
      const maxPlayersInt = maxPlayers
      const winnersInt = 1;
      const durationSecondsInt = 60;
      const uniqueIdentifierInt = 123457;
      const wagerInt = 10_000; 
     

      const mintPublicKey = new PublicKey(tokenMint);
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
      const gameMaker = provider.wallet.publicKey;

      // Generate the game_account PDA with the corrected seeds
      const [gameAccountPDA, gameAccountBump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("GAME"),
          gameMaker.toBuffer(),
          Buffer.from(new Uint32Array([uniqueIdentifierInt]).buffer)
        ],
        program.programId
      );

      const [gameAccountTokenAccount, gameAccountTokenAccountBump] = PublicKey.findProgramAddressSync(
        [
          gameAccountPDA.toBuffer(),
        ],
        program.programId
      );

      // if wager type is 0 then its same wager, if its 1 then its custom wager
      const wager_type = new BN(0);
      // Prepare instruction using the program method
      const instruction = await program.methods.createGame(
        new BN(maxPlayersInt),
        new BN(winnersInt),
        new BN(durationSecondsInt),
        new BN(uniqueIdentifierInt),
        wager_type,
        new BN(wagerInt)
      ).accounts({
        gameAccount: gameAccountPDA,
        mint: mintPublicKey,
        gameAccountTaAccount: gameAccountTokenAccount,
        gameMaker: gameMaker,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      }).instruction();

      // Fetch the latest blockhash
      const block = await provider.connection.getLatestBlockhash('finalized');

      // Create the message for a versioned transaction
      const messageV0 = new TransactionMessage({
        payerKey: provider.wallet.publicKey,
        recentBlockhash: block.blockhash,
        instructions: [instruction],
      }).compileToV0Message();

      // Create the versioned transaction
      const transaction = new VersionedTransaction(messageV0);

      // Sign and send the transaction
      const signedTx = await provider.wallet.signTransaction(transaction);
      const txId = await provider.connection.sendTransaction(signedTx, {skipPreflight: true, preflightCommitment: "confirmed"});

      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error creating game:", error);
    }
  };

  const joinGame = async (game, creatorAddressPubKey: PublicKey, creatorFee, wager) => {
    console.log(game.publicKey, creatorAddressPubKey, creatorFee, wager);
    try {
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
  
      // Here gamePubKey is the PublicKey of the game you're joining
      // creatorAddressPubKey is the PublicKey of the creator for the game
  
      const playerPubKey = provider.wallet.publicKey;
  
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
  
      const block = await provider.connection.getLatestBlockhash('finalized');
  
      const messageV0 = new TransactionMessage({
        payerKey: playerPubKey,
        recentBlockhash: block.blockhash,
        instructions: [instruction],
      }).compileToV0Message();
  
      const transaction = new VersionedTransaction(messageV0);
      const signedTx = await provider.wallet.signTransaction(transaction);
      const txId = await provider.connection.sendTransaction(signedTx, {skipPreflight: false, preflightCommitment: "confirmed"});
      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error joining game:", error);
    }
  };

  const leaveGame = async (game) => {

    try {
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
  
      const playerPubKey = provider.wallet.publicKey;
  
      // Find the associated token accounts for the player and the creator
      const playerAta = getAssociatedTokenAddressSync(
        game.account.mint,
        playerPubKey,
      );

      const [gameAccountTaAccountPDA, gameAccountTaAccountBump] = PublicKey.findProgramAddressSync(
        [game.publicKey.toBuffer()],
        program.programId
      );

      const gameAccountPubKey = (game.publicKey instanceof PublicKey) ? game.publicKey : new PublicKey(game.publicKey);

      const instruction = await program.methods.leaveGame()
        .accounts({
          gameAccount: gameAccountPubKey,
          gameAccountTa: gameAccountTaAccountPDA,
          mint: game.account.mint,
          playerAccount: playerPubKey,
          playerAta: playerAta,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
        }).instruction();
  
      const block = await provider.connection.getLatestBlockhash('finalized');
  
      const messageV0 = new TransactionMessage({
        payerKey: playerPubKey,
        recentBlockhash: block.blockhash,
        instructions: [instruction],
      }).compileToV0Message();
  
      const transaction = new VersionedTransaction(messageV0);
      const signedTx = await provider.wallet.signTransaction(transaction);
      const txId = await provider.connection.sendTransaction(signedTx, {skipPreflight: false, preflightCommitment: "confirmed"});
      console.log("Transaction ID:", txId);
    } catch (error) {
      console.error("Error Leaving game:", error);
    }
  };

  const settleGame = async (game) => {
    try {

      const provider = getProvider();
      const program = new Program(idl, programID, provider);

      const gamba_fee_collector = new PublicKey("5r5Sos7CQUNdN9EpwwSu1ujGVnsChv24TmrtjTWkAdNj")

      const gamba_fee_ata = getAssociatedTokenAddressSync(
        game.account.mint,
        gamba_fee_collector,
      );

      const [gameAccountTaPDA] = PublicKey.findProgramAddressSync(
        [game.publicKey.toBuffer()],
        program.programId
      );
  
      const settleInstruction = await program.methods.settleGame()
        .accounts({
          rng: provider.wallet.publicKey, // just the user for now
          gameMaker: game.account.gameMaker,
          gameAccount: game.publicKey,
          gameAccountTa: gameAccountTaPDA,
          mint: game.account.mint,
          gambaFeeAta: gamba_fee_ata,
          // Player ATAs and Creator ATAs setup
          player1Ata: game.account.players[0] ? game.account.players[0].userAta : null,
          creator1Ata: game.account.players[0] ? game.account.players[0].creatorAddressAta : null,
          player2Ata: game.account.players[1] ? game.account.players[1].userAta : null,
          creator2Ata: game.account.players[1] ? game.account.players[1].creatorAddressAta : null,
          player3Ata: game.account.players[2] ? game.account.players[2].userAta : null,
          creator3Ata: game.account.players[2] ? game.account.players[2].creatorAddressAta : null,
          player4Ata: game.account.players[3] ? game.account.players[3].userAta : null,
          creator4Ata: game.account.players[3] ? game.account.players[3].creatorAddressAta : null,
          // Continue as necessary for all players
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
        }).instruction();
  
      const block = await provider.connection.getLatestBlockhash('finalized');
  
      const messageV0 = new TransactionMessage({
        payerKey: provider.wallet.publicKey,
        recentBlockhash: block.blockhash,
        instructions: [settleInstruction],
      }).compileToV0Message();
  
      const transaction = new VersionedTransaction(messageV0);
      const signedTx = await provider.wallet.signTransaction(transaction);
      const txId = await provider.connection.sendTransaction(signedTx, { skipPreflight: true, preflightCommitment: "confirmed" });
  
      console.log("Settle Game Transaction ID:", txId);
    } catch (error) {
      console.error("Error settling game:", error);
    }
  };
  
  return (
    <div>
      <WalletModalButton />
      <div>Wallet Public Key: {wallet.publicKey?.toString()}</div>
      <div>
        <input
          className="input-field"
          type="number"
          value={maxPlayers}
          onChange={(e) => setMaxPlayers(e.target.value)}
          placeholder="Max Players"
        />
        <input
          className="input-field"
          type="text"
          value={tokenMint}
          onChange={(e) => setTokenMint(e.target.value)}
          placeholder="Token Mint"
        />
        <button onClick={createGame}>Create Game</button>
      </div>
      <div className="button-row">
        <button onClick={fetchGames}>Refresh Games</button>
      </div>
      <div>
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
      </div>
    </div>
  );
}

export default App;