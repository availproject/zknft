'use client';
import React, {
  useState,
  ChangeEventHandler,
  useRef
} from 'react';
import ModalWrapper from './ModalWrapper';
import { NFT } from 'lib/zknft/types';
import { buyNFT, checkPayment } from 'lib/zknft';

interface BuyNftModalProps {
  open: boolean;
  closeModal: () => void;
  nft: NFT,
}

export default function BuyNftModal({
  open,
  closeModal,
  nft,
}: BuyNftModalProps) {
  const [sender, setSender] = useState('');
  const [buyLoading, setBuyLoading] = useState(false);
  const [buyDone, setBuyDone] = useState(false);
  const nftToBuy = nft;

  const handleSend = async () => {
    console.log("Sending")
    if (sender !== '' && (sender.length === 64 || sender.length === 66)) {
      setBuyLoading(true);

      await buyNFT(sender, nftToBuy.id);
      setBuyDone(true);
    }
  }

  const check = async () => {
    console.log("checking", typeof nftToBuy.id);

    console.log("response", await checkPayment(nftToBuy.id));
  }
  return (
    <ModalWrapper
      isOpen={open}
      closeModal={closeModal}
      contentStyle="columns"
      className="h-2/3 w-3/4 md:max-h-[460px] md:max-w-[715px]"
    >
      <section className="relative h-full w-1/3 py-12 px-9 border-r border-[#1E1E24] hidden md:block">
        <h1 className="text-3xl text-white">{nftToBuy.metadata.name}</h1>
        <p className="font-display mt-4 text-white/80 text-sm font-medium">
          {nftToBuy.metadata.description}
        </p>
        <img className="absolute bottom-0 left-0 right-0" src={nftToBuy.metadata.url} />
      </section>
      <section className="h-full w-full md:w-2/3 p-8">
        <h2 className="font-display mt-4 text-white/80 text-sm font-medium">
          Enter your payment address.
        </h2>
        <div className="relative flex font-medium mt-4">
          <input
            value={sender}
            onChange={(e) => setSender(e.target.value)}
            type="text"
            className="p-4 focus:ring-gray-700 focus:outline-none placeholder-[#5B5B65] text-[#5B5B65] focus:border-2 inline-block focus:border-gray-700 w-full sm:text-sm border-white/20 bg-black border rounded-md"
            placeholder="Ex. '0xabcdef...'"
          />
        </div>
        <div className="text-center">
          {!buyDone ? <button onClick={handleSend} disabled={false} className="mt-6 ml-auto mr-auto w-[100px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
            Confirm
          </button> :
            <button onClick={check} disabled={false} className="mt-6 ml-auto mr-auto w-[100px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
              Check
            </button>
          }
        </div>
      </section>
    </ModalWrapper>
  );
}
