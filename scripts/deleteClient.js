import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { PROGRAM, CONNECTION, transaction, WALLET } from "./helper.js";

let bookkeeper = PublicKey.findProgramAddressSync(
  [Buffer.from("bookkeeper")],
  PROGRAM
);

let request = PublicKey.findProgramAddressSync(
  [Buffer.from("request"), WALLET.publicKey.toBuffer()],
  PROGRAM
);

const keys = [
  {
    pubkey: WALLET.publicKey,
    isSigner: true,
    isWritable: true,
  },
  {
    pubkey: bookkeeper[0],
    isSigner: false,
    isWritable: true,
  },
  {
    pubkey: request[0],
    isSigner: false,
    isWritable: true,
  },
];

const data = Buffer.concat([
  //enum for instruction : 2 for deleting client
  Buffer.from(Int8Array.from([2]).buffer),
  // client id
  Buffer.from(Int8Array.from([0]).buffer),
]);

const tx = new TransactionInstruction({
  keys: keys,
  programId: PROGRAM,
  data: data,
});

const sign = await transaction(tx, CONNECTION, WALLET);

console.log("transaction signature is", sign);
