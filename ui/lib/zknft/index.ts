import { NFT, Menu, MenuType } from './types';
import nftImage_1 from '../../public/img/nft-1.jpg';
import nftImage_2 from '../../public/img/nft-2.jpg';
import nftImage_3 from '../../public/img/nft-3.png';
import nftImage_4 from '../../public/img/nft-4.jpg';
import nftImage_5 from '../../public/img/nft-5.jpg';

export async function getForSaleNFTs(): Promise<NFT[]> {
  return [
    {
      id: "1",
      owner: "app",
      nonce: "1",
      metadata: {
        name: "Earth",
        url: nftImage_1,
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
        url: nftImage_4,
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
        url: nftImage_2,
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
        url: nftImage_3,
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
        url: nftImage_5,
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