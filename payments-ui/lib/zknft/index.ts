import * as ed from '@noble/ed25519';
import { TransactionMessage, TransactionCall, $transactionMessage, $signature } from "./types";
import { to_H256, to_H512, hexToAddress, byteArrayToHexString } from "./utils";
import { bytesToHex } from 'web3-utils';
const custodianAddress = [
  110, 80, 211, 15, 198, 63, 39, 13, 44, 74, 228, 84, 127, 23, 174, 86,
  128, 8, 98, 221, 246, 140, 222, 118, 13, 70, 1, 141, 19, 114, 90, 31,
];

export function setLocalStorage(key: string, value: any) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (error) {
    console.error('Error storing data in localStorage:', error);
  }
}

// Retrieve data from localStorage
export function getLocalStorage<T>(key: string): T | null {
  try {
    const storedValue = localStorage.getItem(key);
    return storedValue ? JSON.parse(storedValue) : null;
  } catch (error) {
    console.error('Error retrieving data from localStorage:', error);
    return null;
  }
}

export function getPrivateKey(): Uint8Array {
  let seed = getLocalStorage("my-private-key")
  if (seed === null) {
    const privKey = ed.utils.randomPrivateKey();

    setLocalStorage("my-private-key", Array.from(privKey));

    return privKey;
  } else {
    const privateKey = new Uint8Array(seed as ArrayBufferLike);
    console.log("returning private key.")
    return privateKey;
  }
}

export async function getAddress(): Promise<string> {
  console.log("public", await ed.getPublicKeyAsync(getPrivateKey()));
  return bytesToHex(await ed.getPublicKeyAsync(getPrivateKey()));
}

export async function transfer(to: string, amount: bigint): Promise<void> {
  const privateKey: Uint8Array = getPrivateKey();
  console.log("Got private keyyy.");

  const transactionMessage: TransactionMessage = {
    from: to_H256(Array.from(await ed.getPublicKeyAsync(privateKey))),
    to: hexToAddress(to),
    amount: amount,
    call_type: { CallType: "Transfer" },
    data: undefined,
  };

  let publicKey: Uint8Array = await ed.getPublicKeyAsync(privateKey);

  //MINT
  // const transactionMessage: TransactionMessage = {
  //   from: to_H256(Array.from(await ed.getPublicKeyAsync(privateKey))),
  //   to: to_H256(Array.from(await ed.getPublicKeyAsync(privateKey))),
  //   amount: amount,
  //   call_type: { CallType: "Mint" },
  //   data: undefined,
  // };


  // Convert the JSON string to bytes
  const signature = await ed.signAsync($transactionMessage.encode(transactionMessage), privateKey);
  const isValid = await ed.verifyAsync(signature, $transactionMessage.encode(transactionMessage), publicKey);

  console.log("tx is valid: ", isValid);
  console.log("encoded tx: ", Array.from($transactionMessage.encode(transactionMessage)));
  const transaction: TransactionCall = {
    message: Array.from($transactionMessage.encode(transactionMessage)),
    signature: to_H512(Array.from(signature)),
  }

  const txEndpoint = 'http://127.0.0.1:7001/tx';
  console.log("sending tx.")
  // Create a POST request
  const response = await fetch(txEndpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(transaction),
    cache: 'no-cache',
  })

  if (!response.ok) {
    throw new Error(`Request failed with status: ${response.status}`);
  }

  const responseData = await response.json();

  console.log(responseData);
  return;
}
