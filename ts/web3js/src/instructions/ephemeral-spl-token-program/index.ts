export * from "./ephemeralAta.js";
export * from "./schedulePrivateTransfer.js";
export * from "./transferQueue.js";

export function instructionBytes(discriminator: number): number[] {
  return [discriminator, 0, 0, 0, 0, 0, 0, 0];
}
export function instructionU8Array(discriminator: number): Uint8Array {
  return new Uint8Array([discriminator, 0, 0, 0, 0, 0, 0, 0]);
}
