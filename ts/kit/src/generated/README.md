# Generated Code from IDL (@solana/kit)

This directory contains instruction types and serialization code generated from the delegation program IDL for @solana/kit.

## Generated Instructions

- `instructions/delegate.ts` - Delegate instruction (discriminator: [0,0,0,0,0,0,0,0])
- `instructions/topUpEphemeralBalance.ts` - TopUpEphemeralBalance instruction (discriminator: [9,0,0,0,0,0,0,0])
- `instructions/closeEphemeralBalance.ts` - CloseEphemeralBalance instruction (discriminator: [11,0,0,0,0,0,0,0])

## Adding New Instructions

To add a new instruction from the IDL:

1. Create a new file in `instructions/` directory (e.g., `myInstruction.ts`)
2. Follow the pattern from existing instructions:
   - Export the instruction args type
   - Implement `create{InstructionName}Instruction()` function that returns an `Instruction`
   - Implement `serialize{InstructionName}InstructionData()` function that returns `[Uint8Array]`
3. Export from `instructions/index.ts`

## Note

Code for @solana/kit is manually maintained since Solita doesn't support the @solana/kit API. For web3.js instructions, use the Solita code generation tool from the `web3js/` folder.

## IDL Reference

The IDL is fetched from: `https://raw.githubusercontent.com/magicblock-labs/delegation-program/main/idl/delegation.json`
