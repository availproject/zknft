import Grid from 'components/grid';
import { GridTileImage } from 'components/grid/tile';
import { NFT } from 'lib/zknft/types';
import Link from 'next/link';

export default function ProductGridItems({ products }: { products: NFT[] }) {
  return (
    <>
      {products.map((product) => (
        <Grid.Item key={product.metadata.name} className="animate-fadeIn">
          <Link className="relative inline-block h-full w-full" href={`/product/${product.metadata.name}`}>
            <GridTileImage
              alt={product.metadata.name}
              label={{
                title: product.metadata.name,
                amount: product.price.toLocaleString(),
                currencyCode: product.currencySymbol,
              }}
              src={product.metadata.url}
              fill
              sizes="(min-width: 768px) 33vw, (min-width: 640px) 50vw, 100vw"
            />
          </Link>
        </Grid.Item>
      ))}
    </>
  );
}
