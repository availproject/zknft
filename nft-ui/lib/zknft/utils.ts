import { H256, H512 } from "./types";

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
  const hex: string[] = [];
  byteArray.forEach(byte => {
    hex.push(byte.toString(16).padStart(2, '0'))
  });;

  return hex.join("");
}

export function hexToAddress(hexString: string): H256 {
  if (hexString.length !== 64) {
    throw new Error("Hexadecimal string must be 64 characters (32 bytes) long.");
  }

  // Remove any leading "0x" or "0X" from the hex string
  hexString = hexString.replace(/^0x/i, '');

  // Create an array to store the resulting numbers
  const numberArray = [];

  // Iterate over the hexadecimal string and convert each pair of characters to a number
  for (let i = 0; i < hexString.length; i += 2) {
    const hexPair = hexString.substring(i, 2);
    const decimalNumber = parseInt(hexPair, 16);
    numberArray.push(decimalNumber);
  }

  return to_H256(numberArray);
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
