import {
  PublicKey,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";
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
    pubkey: request[0],
    isSigner: false,
    isWritable: true,
  },
];

let name = "dir-2/file-1";
let nameBuffer = Buffer.from(name);
let zeroesBuffer = Buffer.from(
  Array.from({ length: 128 - nameBuffer.length }, () => 0)
);
let namee = Buffer.concat([nameBuffer, zeroesBuffer]);

let input = Buffer.from("test3");

let zeros = Buffer.from(Array.from({ length: 512 - input.length }, () => 0));

let ip = Buffer.concat([input, zeros]);

const data = Buffer.concat([
  //enum for instruction : 3 for sending requests
  Buffer.from(Int8Array.from([3]).buffer),
  //  client_id
  Buffer.from(Int8Array.from([0]).buffer),
  //enum for request type : 0 for create bucket
  Buffer.from(Int8Array.from([1]).buffer),
  //name as buffer
  namee,
  //file id
  Buffer.from(Int8Array.from([45]).buffer),
  //position
  Buffer.from(Uint8Array.of(...new BN(5).toArray("le", 8))),
  // data
  ip,
]);

const tx = new TransactionInstruction({
  keys: keys,
  programId: PROGRAM,
  data: data,
});

const sign = await transaction(tx, CONNECTION, WALLET);

console.log("transaction signature is", sign);
