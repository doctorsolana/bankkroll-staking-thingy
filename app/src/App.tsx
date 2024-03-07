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
const formatPlayer = (player) => {
  return `Creator ATA: ${formatPublicKey(player.creatorAddressAta)}, User ATA: ${formatPublicKey(player.userAta)}, Creator Fee: ${player.creatorFeeAmount}, Wager: ${player.wager}`;
};

const App = () => {
  const wallet = useWallet();
  const [games, setGames] = useState([]);
  const [maxPlayers, setMaxPlayers] = useState(2); // Default to 2 players
  const [winnerPublicKey, setWinnerPublicKey] = useState('');
  const [tokenMint, setTokenMint] = useState(''); 

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
      const provider = getProvider();
      const program = new Program(idl, programID, provider);
      const gameAccounts = await program.account.game.all();
      console.log("Fetched Games: ", gameAccounts);
      setGames(gameAccounts);
    } catch (error) {
      console.error("Error fetching game accounts:", error);
    }
  };

  const createGame = async () => {
    try {
      const maxPlayersInt = parseInt(maxPlayers, 10); // Ensure maxPlayers is an integer
      if (isNaN(maxPlayersInt) || maxPlayersInt <= 0) {
        alert("Please enter a valid number for max players.");
        return;
      }

      const mintPublicKey = new PublicKey(tokenMint);

      const provider = getProvider();
      const program = new Program(idl, programID, provider);

      const gameMaker = provider.wallet.publicKey;

      // Generate the game_account PDA
      const [gameAccountPDA, gameAccountBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("GAME"), gameMaker.toBuffer(), new Uint8Array(new Uint32Array([maxPlayers]).buffer)],
        program.programId
      );

      // Generate the game_account_ta_account PDA
      // Note: You need to have the `mint` account's PublicKey ready. Let's assume it's available as `mintPublicKey`.
      const [gameAccountTaAccountPDA, gameAccountTaAccountBump] = PublicKey.findProgramAddressSync(
        [gameAccountPDA.toBuffer()],
        program.programId
      );

      // Prepare instruction using the program method
      const instruction = await program.methods.createGame(maxPlayers)
        .accounts({
          gameAccount: gameAccountPDA,
          mint: mintPublicKey,
          gameAccountTaAccount: gameAccountTaAccountPDA,
          gameMaker: provider.wallet.publicKey,
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
      const txId = await provider.connection.sendTransaction(signedTx, {skipPreflight: false, preflightCommitment: "confirmed"});

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
      if (!winnerPublicKey) {
        alert("Please enter a valid public key for the winner.");
        return;
      }
      const provider = getProvider();
      const program = new Program(idl, programID, provider);

      const winnerPublicKeyObj = new PublicKey(winnerPublicKey);
  
      // Assuming game.mint and game.players[x].user_ata are available and correctly set
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
          winnerAta: winnerPublicKeyObj,
          // Player ATAs and Creator ATAs setup
          player1Ata: game.account.players[0] ? game.account.players[0].userAta : null,
          creator1Ata: game.account.players[0] ? game.account.players[0].creatorAddressAta : null,
          player2Ata: game.account.players[1] ? game.account.players[1].userAta : null,
          creator2Ata: game.account.players[1] ? game.account.players[1].creatorAddressAta : null,
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
        {games.map((game, index) => (
          <div key={index} className="gameCard">
            <div>Game Account Public Key: {formatPublicKey(game.publicKey)}</div>
            <div>Game ID: {formatBN(game.account.gameId)}</div>
            <div>State: {parseGameState(game.account.state)}</div>
            <div>Game Maker: {formatPublicKey(game.account.gameMaker)}</div>
            <div>Mint: {formatPublicKey(game.account.mint)}</div>
            <div>Max Players: {game.account.maxPlayers}</div>
            <div>
              Players:
              {game.account.players.map((player, playerIndex) => (
                <div key={playerIndex} className="playerInfo">
                  <div>Creator ATA: {formatPublicKey(player.creatorAddressAta)}</div>
                  <div>User ATA: {formatPublicKey(player.userAta)}</div>
                  <div>Creator Fee: {player.creatorFeeAmount.toString()}</div>
                  <div>Wager: {player.wager.toString()}</div>
                </div>
              ))}
            </div>
            <button onClick={() => joinGame(game, new PublicKey("5r5Sos7CQUNdN9EpwwSu1ujGVnsChv24TmrtjTWkAdNj"), 100, 5000)}>Join Game</button>
            <button onClick={() => leaveGame(game)}>Leave Game</button>
            <input 
              type="text" 
              value={winnerPublicKey} 
              onChange={(e) => setWinnerPublicKey(e.target.value)} 
              placeholder="Winner's ata Key" 
            />
            <button onClick={() => settleGame(game)}>Settle Game</button>
          </div>
        ))}
      </div>
    </div>
  );  
};


export default App;