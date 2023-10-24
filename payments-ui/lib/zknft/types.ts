import * as scale from "subshape";
import { Shape } from "subshape";

export type H256 = [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number,];
export type H512 = [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number,]

export type CallType = { readonly CallType: "Transfer"; } | { readonly CallType: "Mint"; };

export interface TransactionMessage {
  from: H256;
  to: H256;
  amount: bigint;
  call_type: CallType;
  data: string | undefined;
}

export const $address: Shape<H256> = scale.sizedArray(scale.u8, 32);
export const $signature: Shape<H512> = scale.sizedArray(scale.u8, 64);

export const $transactionMessage: Shape<TransactionMessage> = scale.object(
  scale.field("from", $address),
  scale.field("to", $address),
  scale.field("amount", scale.u64),
  scale.field("call_type", scale.taggedUnion(
    "CallType", [
    scale.variant("Transfer"),
    scale.variant("Mint")
  ])),
  scale.field("data", scale.option(scale.str))
);

// TypeScript Interfaces for Transaction and TransactionMessage
export interface Transaction {
  message: scale.Output<typeof $transactionMessage>;
  signature: scale.Output<typeof $signature>;
}

const $transaction: Shape<Transaction> = scale.object(
  scale.field("message", $transactionMessage),
  scale.field("signature", $signature)
)

export interface TransactionCall {
  message: number[],
  signature: H512,
}
