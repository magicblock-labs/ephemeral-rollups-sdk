# Generated Code from IDL (web3.js)

This directory contains instruction types and serialization code generated from the delegation program IDL for web3.js.

## Generated Instructions

- `delegate.ts` - Delegate instruction (discriminator: [0,0,0,0,0,0,0,0])
- `topUpEphemeralBalance.ts` - TopUpEphemeralBalance instruction (discriminator: [9,0,0,0,0,0,0,0])
- `closeEphemeralBalance.ts` - CloseEphemeralBalance instruction (discriminator: [11,0,0,0,0,0,0,0])

## Usage

All instructions follow a similar pattern:

```typescript
import { createDelegateInstruction } from './generated';
import { SystemProgram } from '@solana/web3.js';

const instruction = createDelegateInstruction(
  {
    payer: payerPublicKey,
    delegatedAccount: delegatedAccountKey,
    ownerProgram: ownerProgramKey,
    delegateBuffer: delegateBufferKey,
    delegationRecord: delegationRecordKey,
    delegationMetadata: delegationMetadataKey,
    systemProgram: SystemProgram.programId,
  },
  {
    commitFrequencyMs: 1000,
    seeds: [new Uint8Array([1, 2, 3])],
    validator: validatorPublicKey,
  }
);
```

## Code Generation

To regenerate code from the IDL, run from the `web3js/` folder:

```bash
yarn generate
```

This uses Metaplex Solita with the configuration in `.solitarc.js`.

## IDL Reference

The IDL is fetched from: `https://raw.githubusercontent.com/magicblock-labs/delegation-program/main/idl/delegation.json`
