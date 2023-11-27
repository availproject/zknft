import { NFT, Menu, MenuType, NFTResponseObject, BuyNftQuery, CheckPaymentReply } from './types';
import { hexToAddress, } from "./utils";
import * as ed from '@noble/ed25519';
import { bytesToHex, hexToNumberString } from "web3-utils";
import { sha512 } from '@noble/hashes/sha512';

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

    return privateKey;
  }
}

export function getBuyerAddress(): string {
  ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m));

  let privateKey = getPrivateKey();
  let publicKey: Uint8Array = ed.getPublicKey(privateKey);

  return bytesToHex(publicKey);
}

export async function getForSaleNFTs(): Promise<NFT[]> {
  console.log("get for sale called.")
  const url = 'http://127.0.0.1:7000/listed-nfts/'; // Replace with the actual URL

  try {
    const response = await fetch(url, { cache: 'no-store' });
    const custodian = bytesToHex(Uint8Array.from(custodianAddress));

    if (response.ok) {
      // Successful response, process the data
      const jsonData: NFTResponseObject[] = await response.json();

      let nfts_to_return: NFT[] = [];

      for (const nft of jsonData) {
        nfts_to_return.push(
          {
            id: hexToNumberString(bytesToHex(Uint8Array.from(nft.id))),
            owner: bytesToHex(Uint8Array.from(nft.owner)),
            future: nft.future ? {
              to: bytesToHex(Uint8Array.from(nft.future.to)),
              commitment: nft.future.commitment
            } : undefined,
            nonce: nft.nonce,
            metadata: nft.metadata,
            price: 10,
            currencySymbol: "PVL",
            custodian
          }
        )
      }

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

export async function buyNFT(paymentSender: string, nftId: string): Promise<void> {
  console.log("get for sale called.")
  const url = 'http://127.0.0.1:7000/buy-nft/'; // Replace with the actual URL
  //Safety check
  try {
    parseInt(nftId);
    hexToAddress(paymentSender);
  } catch (error) {
    throw error;
  }

  try {
    let privateKey: Uint8Array = await getPrivateKey();
    let publicKey: Uint8Array = await ed.getPublicKeyAsync(privateKey);
    let hex: string = bytesToHex(publicKey);

    let buyNftQuery: BuyNftQuery = {
      nft_id: nftId,
      payment_sender: paymentSender,
      nft_receiver: hex,
    };

    console.log(buyNftQuery);
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(buyNftQuery),
      cache: 'no-cache',
    });
  } catch (e) {
    console.log(e);
    return;
  }
}

export async function checkPayment(nft_id: string): Promise<CheckPaymentReply> {
  console.log("CHEEECCKKK BROOOOOOO");

  const url = "http://127.0.0.1:7000/check-payment/";

  const response = await fetch(url + nft_id, { cache: 'no-store' });
  console.log(response.body);
  if (response.ok) {
    const status: CheckPaymentReply = await response.json();

    console.log(status);
    return status
  } else {
    throw Error("Request failed with status: " + response.status.toString())
  }
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


// export async function sendTx(): Promise<void> {
//   let nftMetadata: NftMetadata = {
//     name: "ape",
//     url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
//     description: "Demo NFT, not real",
//   }

//   let privateKey: Uint8Array = await getPrivateKey();
//   let publicKey: Uint8Array = await ed.getPublicKeyAsync(privateKey);
//   let publicAddress256: H256 = to_H256(Array.from(publicKey));
//   let hex: string = bytesToHex(publicKey);

//   console.log("original.", publicKey);
//   console.log("hexxx", hex);
//   console.log("array", hexToBytes(hex));

//   let mint: Mint = {
//     from: publicAddress256,
//     to: publicAddress256,
//     data: undefined,
//     future_commitment: undefined,
//     metadata: nftMetadata,
//     id: toBigEndian(BigInt(1)),
//   }

//   let encoded_message = $transactionMessage.encode({
//     NftTransactionMessage: "Mint",
//     ...mint
//   });

//   const signature = await ed.signAsync(encoded_message, privateKey);
//   const isValid = await ed.verifyAsync(signature, encoded_message, publicKey);

//   console.log("tx is validdd: ", isValid);
//   const transaction: NftTransaction = {
//     message: Array.from(encoded_message),
//     signature: to_H512(Array.from(signature)),
//   }

//   // const txEndpoint = 'http://127.0.0.1:7000/tx';
//   // console.log("sending tx.")
//   // // Create a POST request
//   // const response = await fetch(txEndpoint, {
//   //   method: 'POST',
//   //   headers: {
//   //     'Content-Type': 'application/json',
//   //   },
//   //   body: JSON.stringify(transaction),
//   //   cache: 'no-cache',
//   // })

//   // if (!response.ok) {
//   //   throw new Error(`Request failed with status: ${response.status}`);
//   // }

//   // const responseData = await response.json();

//   // console.log(responseData);
//   // return;
// }
