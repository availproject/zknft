import * as scale from "subshape";
import { Shape } from "subshape";

export type H256 = [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number,];
export type H512 = [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number,]

export type NFTResponseObject = {
  id: number[],
  owner: number[],
  future?: string,
  nonce: number,
  metadata: {
    url: string,
    description: string,
    name: string
  }
}

export type NFT = {
  id: number[],
  owner: number[],
  future?: string,
  nonce: number,
  metadata: NftMetadata,
  price: number,
  currency_symbol: string,
};

export type NftMetadata = {
  name: string,
  url: string,
  description: string,
};

export type Menu = {
  title: string;
  path: string;
};

export enum MenuType {
  "main",
  "footer"
}

export interface Mint {
  id: H256;
  from: H256;
  to: H256;
  data: string | undefined;
  future_commitment: H256 | undefined;
  metadata: NftMetadata;
}

export interface Burn {
  id: H256;
  from: H256;
  data: string | undefined;
  future_commitment: H256 | undefined;
}

export interface Trigger {
  id: H256;
  from: H256;
  data?: string | null;
  merkleProof: MerkleProof;
  receipt: TransactionReceipt;
}

export interface Transfer {
  id: H256;
  from: H256;
  to: H256;
  data: string | undefined;
  future_commitment: H256 | undefined;
}

export type MergeValue = { readonly Value: H256 } |
{
  readonly MergeWithZero: {
    base_node: H256,
    zero_bits: H256,
    zero_count: number,
  }
} |
{
  readonly ShortCut: {
    key: H256,
    value: H256,
    height: number,
  }
}

export interface TransactionReceipt {

}

export interface BuyNftQuery {
  nft_id: string,
  payment_sender: string,
  nft_receiver: string,
}

export interface MerkleProof {
  // leaf bitmap, bitmap.get_bit(height) is true means there need a non zero sibling in this height
  leaves_bitmap: H256[],
  // needed sibling node hash
  merkle_path: MergeValue[],
}

export interface TransferEnum extends Transfer {
  NftTransactionMessage: "Transfer",
}

export interface MintEnum extends Mint {
  NftTransactionMessage: "Mint",
}

export interface BurnEnum extends Burn {
  NftTransactionMessage: "Burn",
}


export type NftTransactionMessage = TransferEnum | MintEnum | BurnEnum;

export interface NftTransaction {
  message: number[],
  signature: H512,
}

export const $address: Shape<H256> = scale.sizedArray(scale.u8, 32);
export const $signature: Shape<H512> = scale.sizedArray(scale.u8, 64);

export const $nft_metadata: Shape<NftMetadata> = scale.object(
  scale.field("name", scale.str),
  scale.field("url", scale.str),
  scale.field('description', scale.str)
)

export const $mint: Shape<Mint> = scale.object(
  scale.field("id", $address),
  scale.field("from", $address),
  scale.field("to", $address),
  scale.field("data", scale.option(scale.str)),
  scale.field("future_commitment", scale.option($address)),
  scale.field("metadata", $nft_metadata),
)

export const $transfer: Shape<Transfer> = scale.object(
  scale.field("id", $address),
  scale.field("from", $address),
  scale.field("to", $address),
  scale.field("data", scale.option(scale.str)),
  scale.field("future_commitment", scale.option($address)),
)

export const $burn: Shape<Burn> = scale.object(
  scale.field("id", $address),
  scale.field("from", $address),
  scale.field("data", scale.option(scale.str)),
  scale.field("future_commitment", scale.option($address)),
)

export const $transactionMessage: Shape<NftTransactionMessage> = scale.taggedUnion(
  "NftTransactionMessage", [
  scale.variant("Transfer", $transfer),
  scale.variant("Mint", $mint),
  scale.variant("Burn", $burn)
]);

// export const $buyNft: Shape<BuyNftQuery> = scale.object(
//   scale.field("nft_id", scale.sizedArray(scale.u8, 32)),
//   scale.field("payment_sender", scale.sizedArray(scale.u8, 32)),
//   scale.field("payment_expected_nonce", scale.u64),
//   scale.field("nft_receiver", scale.sizedArray(scale.u8, 32)),
// )


//Payment types 

export type CallType = { readonly CallType: "Transfer"; } | { readonly CallType: "Mint"; };

export interface TransactionMessage {
  from: H256;
  to: H256;
  amount: bigint;
  call_type: CallType;
  data: string | undefined;
}

export const $payTransactionMessage: Shape<TransactionMessage> = scale.object(
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
