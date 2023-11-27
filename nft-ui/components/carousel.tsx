'use client';
import Link from 'next/link';
import { GridTileImage } from './grid/tile';
import { NFT } from 'lib/zknft/types';

export async function Carousel({ nfts }: { nfts: NFT[] }) {
  // Collections that start with `hidden-*` are hidden from the search page.
  const products = nfts;

  if (!products?.length) return null;

  // Purposefully duplicating products to make the carousel loop and not run out of products on wide screens.
  const carouselProducts = [...products, ...products, ...products];

  return (
    <div className=" w-full overflow-x-auto pb-6 pt-1">
      <ul className="flex animate-carousel gap-4">
        {carouselProducts.map((product, i) => (
          <li
            key={`${product.metadata.name.toLowerCase()}${i}`}
            className="relative aspect-square h-[30vh] max-h-[275px] w-2/3 max-w-[475px] flex-none md:w-1/3"
          >
            <Link href={`/?selectedNFT=${product.id}`} className="relative h-full w-full">
              <GridTileImage
                alt={product.metadata.name}
                label={{
                  title: product.metadata.name,
                  amount: product.price.toLocaleString(),
                  currencyCode: product.currencySymbol
                }}
                src={product.metadata.url}
                fill
                sizes="(min-width: 1024px) 25vw, (min-width: 768px) 33vw, 50vw"
              />
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}
