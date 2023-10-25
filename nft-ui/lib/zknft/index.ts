import { NFT, Menu, MenuType, NFTResponseObject, NftMetadata, Mint, H256, $transactionMessage, NftTransaction } from './types';
import { to_H256, to_H512, hexToAddress, byteArrayToHexString, toBigEndian } from "./utils";
import * as ed from '@noble/ed25519';

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

export async function getForSaleNFTs(): Promise<NFT[]> {
  console.log("get for sale called.")
  const url = 'http://127.0.0.1:7000/listed-nfts/'; // Replace with the actual URL

  try {
    const response = await fetch(url, { cache: 'no-store' });

    if (response.ok) {
      // Successful response, process the data
      const jsonData: NFTResponseObject[] = await response.json();

      let nfts_to_return: NFT[] = [];

      for (const nft of jsonData) {
        nfts_to_return.push(
          {
            ...nft,
            price: 10,
            currency_symbol: "PVL"
          }
        )
      }
      // Now you can work with the JSON data
      console.log(nfts_to_return.length);

      return nfts_to_return;
    } else {
      console.error('Request failed with status:', response.status);
      return [];
    }
  } catch (error) {

    console.error('Error in fetch:', error);
    return [];
  }
}

export async function sendTx(): Promise<void> {
  let nftMetadata: NftMetadata = {
    name: "ape",
    url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
    description: "Demo NFT, not real",
  }

  let privateKey: Uint8Array = await getPrivateKey();
  let publicKey: Uint8Array = await ed.getPublicKeyAsync(privateKey);
  let publicAddress256: H256 = to_H256(Array.from(publicKey));

  let mint: Mint = {
    from: publicAddress256,
    to: publicAddress256,
    data: undefined,
    future_commitment: undefined,
    metadata: nftMetadata,
    id: toBigEndian(BigInt(1)),
  }

  console.log("mint ID: ", toBigEndian(BigInt(1)));

  let encoded_message = $transactionMessage.encode({
    NftTransactionMessage: "Mint",
    ...mint
  });

  const signature = await ed.signAsync(encoded_message, privateKey);
  const isValid = await ed.verifyAsync(signature, encoded_message, publicKey);

  console.log("tx is valid: ", isValid);
  const transaction: NftTransaction = {
    message: Array.from(encoded_message),
    signature: to_H512(Array.from(signature)),
  }

  const txEndpoint = 'http://127.0.0.1:7000/tx';
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

export async function getMenu(type: MenuType): Promise<Menu[]> {
  if (type == MenuType.main) {
    return [
      {
        title: "Home",
        path: "/"
      },
      {
        title: "About",
        path: "/about"
      }
    ]
  } else {
    return [
      {
        title: "Home",
        path: "/"
      },
      {
        title: "About",
        path: "/about"
      }
    ];
  }
}