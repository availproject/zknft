import { NFT, Menu, MenuType } from './types';
import axios from 'axios';

export async function getForSaleNFTs(): Promise<NFT[]> {
  const url = 'http://127.0.0.1:7000/listed-nfts'; // Replace with the actual URL

  try {
    const response = await fetch(url);

    if (response.ok) {
      // Successful response, process the data
      const jsonData = await response.json();

      // Now you can work with the JSON data
      console.log('Listed NFTs:', jsonData);
    } else {
      console.error('Request failed with status:', response.status);
    }
  } catch (error) {
    console.error('Error in fetch:', error);
  }

  return [
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Earth",
        url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
      },
      price: 10,
      currency_symbol: "PVL"
    },
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Panda",
        url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
      },
      price: 10,
      currency_symbol: "PVL"
    },
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Sky",
        url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
      },
      price: 10,
      currency_symbol: "PVL"
    },
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Space",
        url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
      },
      price: 10,
      currency_symbol: "PVL"
    },
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Today",
        url: "https://storage.googleapis.com/nftimagebucket/tokens/0x60e4d786628fea6478f785a6d7e704777c86a7c6/preview/5933.png",
      },
      price: 10,
      currency_symbol: "PVL"
    },
  ];
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