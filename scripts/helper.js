import {
  PublicKey,
  clusterApiUrl,
  Connection,
  Keypair,
  TransactionInstruction,
  Transaction,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import BN from "bn.js";

const key = [
  190, 193, 120, 250, 149, 178, 94, 52, 206, 110, 89, 151, 89, 109, 26, 49, 186,
  112, 209, 9, 158, 240, 185, 6, 106, 26, 47, 222, 57, 110, 24, 236, 71, 78,
  122, 183, 125, 16, 86, 144, 130, 239, 169, 17, 208, 51, 63, 228, 133, 110, 28,
  58, 26, 7, 214, 63, 70, 95, 8, 131, 251, 143, 119, 143,
];

export const WALLET = Keypair.fromSecretKey(Uint8Array.from(key));

export const CONNECTION = new Connection(clusterApiUrl("devnet"));

export const PROGRAM = new PublicKey(
  "ALFYRwSZYXC31JpfSr2yKJ2aHBkbAQ7JXkGydnP3bxrN"
);

export async function transaction(tx, connection, wallet) {
  const ins = ComputeBudgetProgram.setComputeUnitLimit({
    units: 600000,
  });
  const ix = new Transaction().add(ins).add(tx);
  ix.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  ix.feePayer = wallet.publicKey;
  ix.sign(wallet);
  const sign = await connection.sendRawTransaction(ix.serialize());
  await connection.confirmTransaction(sign, "confirmed");

  return sign;
}
