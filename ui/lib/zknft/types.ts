import { StaticImageData } from "next/image";

export type NFT = {
  id: string,
  owner: string,
  future?: string,
  nonce: string,
  metadata: NFTMetadata,
  price: number,
  currency_symbol: string,
};

export type NFTMetadata = {
  name: string,
  url: StaticImageData,
};

export type Menu = {
  title: string;
  path: string;
};

export enum MenuType {
  "main",
  "footer"
}
