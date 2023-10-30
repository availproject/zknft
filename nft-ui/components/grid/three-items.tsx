'use client';
import { GridTileImage } from 'components/grid/tile';
import type { NFT } from 'lib/zknft/types';
import Link from 'next/link';

function ThreeItemGridItem({
  item,
  size,
  priority,
  onClick,
}: {
  item: NFT;
  size: 'full' | 'half';
  priority?: boolean;
  onClick: (nft: NFT) => void;
}) {
  return (
    <div
      className={size === 'full' ? 'md:col-span-4 md:row-span-2' : 'md:col-span-2 md:row-span-1'}
    >
      <div className="relative block aspect-square h-full w-full cursor-pointer">
        <GridTileImage
          src={item.metadata.url}
          fill
          sizes={
            size === 'full' ? '(min-width: 768px) 66vw, 100vw' : '(min-width: 768px) 33vw, 100vw'
          }
          priority={priority}
          alt={item.metadata.name}
          onClick={() => onClick(item)}
          label={{
            position: size === 'full' ? 'center' : 'bottom',
            title: item.metadata.name,
            amount: item.price.toLocaleString(),
            currencyCode: item.currency_symbol,
          }}
        />
      </div>
    </div>
  );
}

export async function ThreeItemGrid({
  buyNft,
  featuredNFTs,
}: {
  buyNft: (nft: NFT) => void;
  featuredNFTs: NFT[]
}) {
  const onBuyNft = buyNft;
  // Collections that start with `hidden-*` are hidden from the search page.
  const homepageItems = featuredNFTs;

  if (!homepageItems[0] || !homepageItems[1] || !homepageItems[2]) return null;

  const [firstProduct, secondProduct, thirdProduct] = homepageItems;

  return (
    <section className="mx-auto grid max-w-screen-2xl gap-4 px-4 pb-4 md:grid-cols-6 md:grid-rows-2">
      <ThreeItemGridItem onClick={onBuyNft} size="full" item={firstProduct} priority={true} />
      <ThreeItemGridItem onClick={onBuyNft} size="half" item={secondProduct} priority={true} />
      <ThreeItemGridItem onClick={onBuyNft} size="half" item={thirdProduct} />
    </section>
  );
}
