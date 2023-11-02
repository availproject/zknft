import { Carousel } from 'components/carousel';
import { ThreeItemGrid } from 'components/grid/three-items';
import Footer from 'components/layout/footer';
import { Suspense, useRef, useState } from 'react';
import BuyNftModal from '../components/modals/BuyNftModal';
import { getForSaleNFTs } from 'lib/zknft/index';
import { headers } from 'next/headers';
import { NFT } from 'lib/zknft/types';

export default async function HomePage({
  searchParams
}: {
  searchParams: { [key: string]: string | string[] | undefined }
}) {
  const nfts = await getForSaleNFTs();

  const selectedNFTId = searchParams.selectedNFT?.toString() ?? null;

  const selectedNFT = nfts.find((nft) => nft.id === selectedNFTId) || null;

  return (
    <>
      <ThreeItemGrid featuredNFTs={nfts} />
      <Suspense>
        <Carousel nfts={nfts} />
        <Suspense>
          <Footer />
        </Suspense>
      </Suspense>
      {selectedNFT !== null && <BuyNftModal open={selectedNFT !== null} nft={selectedNFT as NFT} />}
    </>
  );
}
