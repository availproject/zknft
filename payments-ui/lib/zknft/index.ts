import * as ed from '@noble/ed25519';
import { TransactionMessage, TransactionCall, $transactionMessage, $signature } from "./types";
import { to_H256, to_H512, hexToAddress, byteArrayToHexString } from "./utils";

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

export async function getPrivateKey(): Promise<Uint8Array> {
  let seed = getLocalStorage("my-private-key")

  if (seed === null) {
    const privKey = ed.utils.randomPrivateKey();

    setLocalStorage("my-private-key", Array.from(privKey));

    console.log(await ed.getPublicKeyAsync(privKey));

    return privKey;
  } else {
    const privateKey = new Uint8Array(seed as ArrayBufferLike);

    return privateKey;
  }
}

export async function init(): Promise<void> {
  const privateKey: Uint8Array = await getPrivateKey();
  console.log("Got private keyyy.");
  let publicKey: Uint8Array = await ed.getPublicKeyAsync(privateKey);
  console.log("address: ", byteArrayToHexString(publicKey), byteArrayToHexString(publicKey).length)
}

export async function transfer(to: string, amount: bigint): Promise<void> {
  const privateKey: Uint8Array = await getPrivateKey();
  console.log("Got private keyyy.");

  const transactionMessage: TransactionMessage = {
    from: to_H256(Array.from(await ed.getPublicKeyAsync(privateKey))),
    to: hexToAddress(to),
    amount: amount,
    call_type: { CallType: "Mint" },
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
