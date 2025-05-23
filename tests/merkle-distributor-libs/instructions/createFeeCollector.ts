/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import {
  CreateFeeCollectorArgs,
  createFeeCollectorArgsBeet,
} from '../types/CreateFeeCollectorArgs'

/**
 * @category Instructions
 * @category CreateFeeCollector
 * @category generated
 */
export type CreateFeeCollectorInstructionArgs = {
  args: CreateFeeCollectorArgs
}
/**
 * @category Instructions
 * @category CreateFeeCollector
 * @category generated
 */
export const createFeeCollectorStruct = new beet.BeetArgsStruct<
  CreateFeeCollectorInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', createFeeCollectorArgsBeet],
  ],
  'CreateFeeCollectorInstructionArgs'
)
/**
 * Accounts required by the _createFeeCollector_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] feeCollector
 * @property [] associatedTokenProgram
 * @category Instructions
 * @category CreateFeeCollector
 * @category generated
 */
export type CreateFeeCollectorInstructionAccounts = {
  signer: web3.PublicKey
  feeCollector: web3.PublicKey
  systemProgram?: web3.PublicKey
  tokenProgram?: web3.PublicKey
  associatedTokenProgram: web3.PublicKey
  rent?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const createFeeCollectorInstructionDiscriminator = [
  24, 226, 43, 173, 141, 41, 172, 151,
]

/**
 * Creates a _CreateFeeCollector_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category CreateFeeCollector
 * @category generated
 */
export function createCreateFeeCollectorInstruction(
  accounts: CreateFeeCollectorInstructionAccounts,
  args: CreateFeeCollectorInstructionArgs,
  programId = new web3.PublicKey('AZMc26abaSP7si1wtLaV5yPxTxpWd895M8YpJFFdQ8Qw')
) {
  const [data] = createFeeCollectorStruct.serialize({
    instructionDiscriminator: createFeeCollectorInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.signer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.feeCollector,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.associatedTokenProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rent ?? web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
  ]

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc)
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
