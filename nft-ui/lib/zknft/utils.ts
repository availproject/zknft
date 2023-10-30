import { H256, H512 } from "./types";
import { hexToBytes, bytesToHex } from "web3-utils";

export function to_H256(array: number[]): H256 {
  const h256: H256 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

  for (let i = 0; i < 32; i++) {
    h256[i] = array[i] ?? 0;
  }

  return h256;
}

export function to_H512(array: number[]): H512 {
  const h512: H512 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

  for (let i = 0; i < 64; i++) {
    h512[i] = array[i] as number;
  }

  return h512;
}

export function byteArrayToHexString(byteArray: Uint8Array): string {
  return bytesToHex(byteArray);
}

export function hexToAddress(hexString: string): H256 {
  let bytes = hexToBytes(hexString);

  return to_H256(Array.from(bytes));
}

export function toLittleEndian(bigNumber: bigint): H256 {
  let result: number[] = [];
  let i = 0;

  while (bigNumber > BigInt(0)) {
    result[i] = parseInt((bigNumber % BigInt(256)).toString());

    bigNumber = bigNumber / BigInt(256);
    i += 1;
  }

  return to_H256(Array.from(result));
}

export function toBigEndian(bigNumber: bigint): H256 {
  return to_H256(toLittleEndian(bigNumber).reverse());
}
