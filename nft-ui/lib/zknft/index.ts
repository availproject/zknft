import { NFT, Menu, MenuType, NFTResponseObject } from './types';
import axios from 'axios';

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