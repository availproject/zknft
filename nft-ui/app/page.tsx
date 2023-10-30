'use client';
import { Carousel } from 'components/carousel';
import { ThreeItemGrid } from 'components/grid/three-items';
import Footer from 'components/layout/footer';
import { Suspense, useRef, useState } from 'react';
import BuyNftModal from '../components/modals/BuyNftModal';
import { getForSaleNFTs } from 'lib/zknft/index';
import { NFT } from 'lib/zknft/types';


export default async function HomePage() {
  const [selectedNFT, setSelectedNFT] = useState<NFT | null>(null);
  const nfts = await getForSaleNFTs();

  const closeModal = () => {
    setSelectedNFT(null);
  };

  const openModal = (nft: NFT) => {
    setSelectedNFT(nft);
  }

  return (
    <>
      <ThreeItemGrid buyNft={openModal} featuredNFTs={nfts} />
      <Suspense>
        <Carousel nfts={nfts} />
        <Suspense>
          <Footer />
        </Suspense>
      </Suspense>
      {selectedNFT !== null && <BuyNftModal open={selectedNFT !== null} closeModal={closeModal} nft={selectedNFT as NFT} />}
    </>
  );
}
