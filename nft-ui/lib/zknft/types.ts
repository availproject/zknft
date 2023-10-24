export type NFTResponseObject = {
  id: string,
  owner: [number],
  future?: string,
  nonce: number,
  metadata: {
    url: string,
    description: string,
    name: string
  }
}

export type NFT = {
  id: string,
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
};

export type Menu = {
  title: string;
  path: string;
};

export enum MenuType {
  "main",
  "footer"
}

// type NftId = {
//   value: string; // Assuming U256 is a string representation
// };

// type H256 = number[];

// type Address = number[];

// interface Mint {
//   id: NftId;
//   from: Address;
//   to: Address;
//   data?: string | null;
//   futureCommitment?: H256 | null;
//   metadata: NftMetadata;
// }

// interface Burn {
//   id: NftId;
//   from: Address;
//   data?: string | null;
//   futureCommitment?: H256 | null;
// }

// interface Trigger {
//   id: NftId;
//   from: Address;
//   data?: string | null;
//   merkleProof: MerkleProof;
//   receipt: TransactionReceipt;
// }

// export type NftTransactionMessage = {
//   Transfer: Transfer;
//   Mint: Mint;
//   Burn: Burn;
//   Trigger: Trigger;
// };

// interface Transfer {
//   id: NftId;
//   from: Address;
//   to: Address;
//   data?: string | null;
// }
