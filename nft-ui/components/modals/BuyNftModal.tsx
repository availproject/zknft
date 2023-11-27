'use client';
import React, {
  useState,
  ChangeEventHandler,
  useRef,
  useEffect
} from 'react';
import ModalWrapper from './ModalWrapper';
import { NFT, TransferStatus } from 'lib/zknft/types';
import { buyNFT, checkPayment, getLocalStorage, getPrivateKey } from 'lib/zknft';
import { bytesToHex } from "web3-utils";
import { useParams, usePathname, useRouter, useSearchParams } from 'next/navigation';
import { getBuyerAddress } from 'lib/zknft';
import { useUrl } from 'nextjs-current-url';
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faUser,
  faCopy,
} from "@fortawesome/free-solid-svg-icons";

interface BuyNftModalProps {
  open: boolean;
  nft: NFT,
}

export default async function BuyNftModal({
  open,
  nft,
}: BuyNftModalProps) {
  const [buyer, setBuyer] = useState('');
  const [nftStatus, setNftStatus] = useState<TransferStatus | null>(null);
  //const [paymentAttemptDone, setPaymentAttemptDone] = useState(false);
  const nftToBuy = nft;
  const router = useRouter();

  const closeModal = () => {
    // Close the modal and remove the query parameters from the URL
    router.replace('/');
  };
  const params = useSearchParams();
  const paymentAddress: string | null = params.get("selectedPaymentAddress") as string || null;
  const paymentAttemptDone: string | null = params.get("paymentDone");
  const currentUrl = paymentAddress && paymentAddress !== '' ? `http://localhost:3000/?selectedNFT='${nft.id}&selectedPaymentAddress=${paymentAddress}'` : `http://localhost:3000/?selectedNFT='${nft.id}'`;
  console.log(currentUrl);
  const handleSend = async () => {
    if (paymentAddress) {
      await buyNFT(paymentAddress, nftToBuy.id);
    }

    check();
  }
  const makePayment = () => {
    if (!paymentAddress) {
      alert("Something not right, refresh and try.");
    }

    router.push(`http://localhost:3001/?to=${nft.custodian}&from=${paymentAddress}&amount=${nft.price}&origin='${encodeURIComponent(currentUrl + "&paymentDone=true")}'`)
  }

  const check = () => {
    checkPaymentAndSet();

    router.refresh();
  }

  const checkPaymentAndSet = () => {
    checkPayment(nft.id).then((reply) => setNftStatus(reply.status));
  };

  useEffect(() => {
    console.log("use effect triggered....");
    setBuyer(
      getBuyerAddress()
    );

    checkPaymentAndSet();
  }, []);

  function ModalContent() {
    console.log(nftStatus);
    if (nftStatus === null) return (
      <div className='flex h-full justify-center items-center'>
        Loading hold on....
      </div>
    )
    else if (nftStatus === TransferStatus.HoldInProgress) return (
      <div className='flex flex-col h-full justify-center items-center'>
        <h1 className='text-2xl text-white'>
          Your NFT is being reserved.
        </h1>
        <button onClick={check} disabled={false} className="mt-6 ml-auto mr-auto w-[150px] h-[50px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
          Check if done!
        </button>
      </div>
    )
    else if (nftStatus == TransferStatus.WaitingForPayment) return (
      <div className='flex h-full justify-center items-center'>
        {
          paymentAttemptDone ?
            <div className='flex flex-col gap-4'>
              <button onClick={check} disabled={false} className="mt-6 ml-auto mr-auto w-[100px] h-[50px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                Check
              </button>

              {
                !paymentAddress ? (
                  <a href={`http://localhost:3001/select_address/?origin=${encodeURIComponent(currentUrl)}`}>
                    <button className="w-[150px] h-[40px] bg-gray-800/50 hover:bg-gray-900 text-white py-2 px-4 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                      Try Again.
                    </button>
                  </a>
                ) :
                  <button onClick={makePayment} className="w-[150px] h-[50px] bg-gray-800/50 hover:bg-gray-900 text-white py-2 px-4 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                    Try Again.
                  </button>
              }
            </div>
            : (
              !paymentAddress ? (
                <a href={`http://localhost:3001/select_address/?origin=${encodeURIComponent(currentUrl)}`}>
                  <button className="w-[200px] h-[50px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                    Select Payment.
                  </button>
                </a>
              ) :
                <button onClick={makePayment} className="w-[200px] h-[50px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                  Make Payment.
                </button>
            )
        }
      </div>
    )
    else if ((nftStatus === TransferStatus.NotInitiated) && paymentAddress) return (
      <div className='flex flex-col h-full justify-center items-center'>
        <div className="w-full">
          <p className="flex w-full justify-between border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:w-auto lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
            <span className="flex items-center text-ellipsis overflow-hidden">
              <FontAwesomeIcon
                className="mr-2"
                icon={faUser}
                style={{ fontSize: 14, color: "white" }}
              />
              {paymentAddress}
            </span>
          </p>
        </div>
        <button onClick={handleSend} disabled={false} className="mt-6 ml-auto mr-auto w-[100px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
          Buy now!
        </button>
      </div>
    )
    else if (nftStatus === TransferStatus.NotInitiated) return (
      <div className='flex flex-col justify-center items-center h-full'>
        <a href={`http://localhost:3001/select_address/?origin=${currentUrl}`}>
          <button className="w-[200px] h-[50px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
            Select Payment.
          </button>
        </a>
      </div>
    )
    else if (nftStatus === TransferStatus.PaymentDone && nft.owner === buyer) return (
      <div className='flex flex-col justify-center items-center h-full'>
        It is all yours!
      </div>
    )
    else if (nftStatus === TransferStatus.TransferInProgress) return (
      <div className='flex flex-col justify-center items-center h-full'>
        Hold on will soon be yours.
      </div>
    )
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
        <ModalContent />
      </section>
    </ModalWrapper>
  );
}
