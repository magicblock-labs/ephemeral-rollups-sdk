import { Address, getAddressEncoder } from "@solana/kit";
import { blake2b } from "@noble/hashes/blake2b";
import { edwardsToMontgomeryPub } from "@noble/curves/ed25519";
import * as nacl from "tweetnacl";

export function encryptEd25519Recipient(
  plaintext: Uint8Array,
  recipient: Address,
): Buffer {
  const recipientBytes = getAddressEncoder().encode(recipient);
  const recipientX25519 = edwardsToMontgomeryPub(
    new Uint8Array(recipientBytes),
  );
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
