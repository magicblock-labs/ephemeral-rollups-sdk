import { getMemberDecoder, getMemberEncoder, Member } from "./member";
import {
  getStructEncoder,
  getU8Encoder,
  getAddressDecoder,
  getAddressEncoder,
  getStructDecoder,
  getU8Decoder,
  getOptionEncoder,
  getArrayEncoder,
  getOptionDecoder,
  getArrayDecoder,
  Encoder,
  combineCodec,
  Codec,
  Decoder,
  Address,
  Option,
} from "@solana/kit";

export interface Permission {
  discriminator: number;
  bump: number;
  permissionedAccount: Address;
  members: Option<Member[]>;
}

export function getPermissionEncoder(): Encoder<Permission> {
  return getStructEncoder([
    ["discriminator", getU8Encoder()],
    ["bump", getU8Encoder()],
    ["permissionedAccount", getAddressEncoder()],
    ["members", getOptionEncoder(getArrayEncoder(getMemberEncoder()))],
  ]);
}

export function getPermissionDecoder(): Decoder<Permission> {
  return getStructDecoder([
    ["discriminator", getU8Decoder()],
    ["bump", getU8Decoder()],
    ["permissionedAccount", getAddressDecoder()],
    ["members", getOptionDecoder(getArrayDecoder(getMemberDecoder()))],
  ]);
}

export function getPermissionCodec(): Codec<Permission, Permission> {
  return combineCodec(getPermissionEncoder(), getPermissionDecoder());
}
