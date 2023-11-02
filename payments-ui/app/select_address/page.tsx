'use client';
import Image from 'next/image';
import { transfer, getAddress, setLocalStorage } from 'lib/zknft';
import React, { useState, ChangeEvent } from 'react';
// import Font Awesome CSS
import "@fortawesome/fontawesome-svg-core/styles.css";
import { config } from "@fortawesome/fontawesome-svg-core";
// Tell Font Awesome to skip adding the CSS automatically 
// since it's already imported above
config.autoAddCss = false;
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faUser,
  faCopy,
} from "@fortawesome/free-solid-svg-icons";
import { useRouter, useSearchParams } from 'next/navigation';

export default function SelectAddress() {
  const [selectedAddress, setSelectedAddress] = useState('');
  const router = useRouter();
  const searchParams = useSearchParams();
  const origin: string | null = searchParams.get("origin") || null;
  const originURL = decodeURIComponent(origin as string);
  const urlWithoutQuotes = originURL.replace(/'/g, '');

  const handleSend = () => {
    if (!selectedAddress || !origin) {
      alert("Select address.");
    }
    router.push(`${urlWithoutQuotes}&selectedPaymentAddress=${selectedAddress}`);
  };

  const handleLoadClick = async () => {
    setSelectedAddress(await getAddress());
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm lg:flex">
        <p className="fixed left-0 top-0 flex w-full justify-center border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:static lg:w-auto  lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
          Superfast&nbsp;
          <code className="font-mono font-bold">Zk verified payments.</code>
        </p>
        <div className="fixed bottom-0 left-0 flex h-48 w-full items-end justify-center bg-gradient-to-t from-white via-white dark:from-black dark:via-black lg:static lg:h-auto lg:w-auto lg:bg-none">
          <a
            className="pointer-events-none flex place-items-center gap-2 p-8 lg:pointer-events-auto lg:p-0"
            href="https://www.availproject.org/"
            target="_blank"
            rel="noopener noreferrer"
          >
            By{' '}
            <Image
              src="/avail_logo_white.png"
              alt="Avail Logo"
              width={180}
              height={60}
              priority
            />
          </a>
        </div>
      </div>
      <div className="w-[560px] h-[200px] flex flex-col justify-center">
        {
          selectedAddress ?
            <>
              <div className="relative flex place-items-center before:absolute before:h-[300px] before:w-[480px] before:-translate-x-1/2 before:rounded-full">
                <h2 className={`mb-3 text-2xl font-semibold`}>
                  Select address!
                </h2>
              </div>

              <div
                placeholder="abcd..."
                className="w-full text-ellipsis overflow-hidden text-sm text-white relative rounded-lg border bg-transparent p-4 placeholder:text-neutral-500 border-neutral-500"
              > {selectedAddress}
              </div>
              <button onClick={handleSend} disabled={!selectedAddress} className="mt-10 ml-auto mr-auto w-[100px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                Confirm
              </button>
            </> :
            <div className="flex min-w-full justify-between border-b border-gray-300 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:w-auto lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
              <span className="flex items-center">
                <FontAwesomeIcon
                  className="mr-2"
                  icon={faUser}
                  style={{ fontSize: 14, color: "white" }}
                />
              </span>
              <button onClick={handleLoadClick} className="w-[200px] bg-gray-800 hover:bg-gray-900 text-white font-semibold py-2 px-4  rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
                Load account
              </button>
            </div>
        }
      </div>
      <div className="mb-32 grid text-center lg:max-w-5xl lg:w-full lg:mb-0 lg:grid-cols-4 lg:text-left">
        <a
          href="https://docs.availproject.org/"
          className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30"
          target="_blank"
          rel="noopener noreferrer"
        >
          <h2 className={`mb-3 text-2xl font-semibold`}>
            Docs{' '}
            <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
              -&gt;
            </span>
          </h2>
          <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>
            Find in-depth information about Avail features. Build and power your vision with Avail.
          </p>
        </a>

        <a
          href="https://docs.availproject.org/networks/"
          className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30"
          target="_blank"
          rel="noopener noreferrer"
        >
          <h2 className={`mb-3 text-2xl font-semibold`}>
            Join{' '}
            <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
              -&gt;
            </span>
          </h2>
          <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>
            Our testnet is live. Try it now!
          </p>
        </a>

        <a
          href="https://docs.availproject.org/about/explorer/"
          className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30"
          target="_blank"
          rel="noopener noreferrer"
        >
          <h2 className={`mb-3 text-2xl font-semibold`}>
            Explorer{' '}
            <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
              -&gt;
            </span>
          </h2>
          <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>
            Explore the testnet with the testnet explorer.
          </p>
        </a>

        <a
          href="https://blog.availproject.org/"
          className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30"
          target="_blank"
          rel="noopener noreferrer"
        >
          <h2 className={`mb-3 text-2xl font-semibold`}>
            Blog{' '}
            <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
              -&gt;
            </span>
          </h2>
          <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>
            Stay up to date with all things Avail.
          </p>
        </a>
      </div>
    </main>
  )
}
