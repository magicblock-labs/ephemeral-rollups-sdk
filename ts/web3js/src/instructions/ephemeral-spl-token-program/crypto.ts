import { PublicKey } from "@solana/web3.js";
import { blake2b } from "@noble/hashes/blake2b";
import { edwardsToMontgomeryPub } from "@noble/curves/ed25519";
import * as nacl from "tweetnacl";

//
// This constant defines the total overhead of sealed-box encryption,
// and this value is the sum of the following two constants defined
// by the library.
//
//  - nacl.box.publicKeyLength (32)
//  - nacl.box.overheadLength (16)
//
// Do not confuse `publicKeyLength` with the second argument `recipient` of
// the following encryptWithEd25519Recipient(). It is actually the pubkey
// internally generated and added by the sealed-box encryption algorithn so
// that recipient could send encrypted replies to the original sender.
//
export const ENCRYPTION_OVERHEAD = 48;

export function encryptWithEd25519Recipient(
  plaintext: Uint8Array,
  recipient: PublicKey,
): Buffer {
  const recipientX25519 = edwardsToMontgomeryPub(recipient.toBytes());
  const ephemeral = nacl.box.keyPair();
  const nonce = blake2b(
    Buffer.concat([
      Buffer.from(ephemeral.publicKey),
      Buffer.from(recipientX25519),
    ]),
    { dkLen: nacl.box.nonceLength },
  );
  const ciphertext = nacl.box(
    plaintext,
    nonce,
    recipientX25519,
    ephemeral.secretKey,
  );

  return Buffer.concat([
    Buffer.from(ephemeral.publicKey),
    Buffer.from(ciphertext),
  ]);
}
