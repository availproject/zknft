'use client';
import Image from 'next/image';
import { transfer, getAddress } from 'lib/zknft';
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

export default function Home() {
  const [amount, setAmount] = useState<null | number>(null);
  const [to, setTo] = useState('');
  const router = useRouter();
  const [myValue, setMyValue] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const searchParams = useSearchParams();
  const preSetTo: string | null = searchParams.get("to");
  const preSetAmount: string | null = searchParams.get("amount");
  const preSetFrom: string | null = searchParams.get("from");
  const origin: string | null = searchParams.get("origin") || null;
  const originURL = decodeURIComponent(origin as string);
  const urlWithoutQuotes = originURL.replace(/'/g, '');

  const amountOk = (): boolean => {
    if (amount != 0 && amount != null) {
      return true;
    }

    if (preSetAmount != null && parseInt(preSetAmount) != 0) {
      return true;
    }

    return false;
  }

  const toOk = (): boolean => {
    if (to !== '' && (to.length === 64 || to.length === 66)) {
      return true;
    }

    if (preSetTo && preSetTo !== '' && (preSetTo.length === 64 || preSetTo.length === 66)) {
      return true;
    }

    return false;
  }

  const handleAmountChange = (event: ChangeEvent<HTMLInputElement>) => {
    const amount = parseInt(event.target.value);
    setAmount(parseInt(event.target.value));

    // if (amount != 0 || amount != null) {
    //   setAmountOk(true)
    // }
  };

  const handleToChange = (event: ChangeEvent<HTMLInputElement>) => {
    const to = event.target.value.toLowerCase();
    setTo(to);

    // if (to !== '' && (to.length === 64 || to.length === 66)) {
    //   setToOk(true);
    // }
  };

  const handleSend = () => {
    if (preSetFrom && preSetFrom !== myValue) {
      alert!("From does not match your account.");

      return;
    }
    console.log(amountOk(), toOk());
    if (!amountOk() || !toOk()) {
      return;
    }

    const selectedAmount = preSetAmount || amount as number | string;
    const selectedTo = preSetTo && preSetTo !== '' ? preSetTo : to;

    transfer(selectedTo, BigInt(selectedAmount)).then(() => {
      if (origin) {
        console.log(originURL, origin)
        console.log(urlWithoutQuotes);
        router.push(`${urlWithoutQuotes}`);
      }
    });
  };

  const handleCopyClick = () => {
    if (myValue) {
      navigator.clipboard.writeText(myValue);
      setCopied(true);
      setTimeout(() => setCopied(false), 3000); // Reset copied state after 3 seconds
    }
  };

  const handleLoadClick = async () => {
    setMyValue(await getAddress());
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
      <div className="mt-8 w-full max-w-2xl">
        <p className="flex min-w-full justify-between border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:w-auto lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
          <span className="flex items-center">
            <FontAwesomeIcon
              className="mr-2"
              icon={faUser}
              style={{ fontSize: 14, color: "white" }}
            />
            {myValue ? (
              <code className="font-mono text-xs font-bold">{myValue}</code>
            ) : (
              ''
            )}
          </span>
          {myValue ? (
            <button onClick={handleCopyClick} className="bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
              {copied ? 'Copied!' : <FontAwesomeIcon
                icon={faCopy}
                style={{ fontSize: 14, color: "white" }}
              />}
            </button>
          ) : (
            <button onClick={handleLoadClick} className="w-[200px] bg-gray-800 hover:bg-gray-900 text-white font-semibold py-2 px-4  rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
              Load account
            </button>
          )}
        </p>
      </div>

      <div className="w-[560px] h-[200px] flex flex-col justify-center before:bg-gradient-radial before:from-white before:to-transparent before:blur-2xl before:content-[''] after:absolute after:h-[180px] after:w-[240px] after:translate-x-2/3 after:bg-gradient-conic after:from-sky-200 after:via-blue-200 after:blur-2xl after:content-[''] before:dark:bg-gradient-to-br before:dark:from-transparent before:dark:to-blue-700 before:dark:opacity-10 after:dark:from-sky-900 after:dark:via-[#0141ff] after:dark:opacity-40 before:lg:h-[250px]">
        <div className="relative flex place-items-center before:absolute before:h-[300px] before:w-[480px] before:-translate-x-1/2 before:rounded-full">
          <h2 className={`mb-3 text-2xl font-semibold`}>
            Send money now!
          </h2>
        </div>

        <div className="relative flex place-items-center before:absolute before:h-[100px] before:w-[560px] before:-translate-x-1/2 before:rounded-full ">
          {preSetTo ? <input
            type="text"
            name="transfer-to"
            placeholder="abcd..."
            autoComplete="off"
            disabled={true}
            value={preSetTo}
            onChange={handleToChange}
            className="w-full text-ellipsis overflow-hidden text-xs relative rounded-lg border bg-white pl-4 pr-8 py-2  text-sm text-black placeholder:text-neutral-500 dark:border-neutral-800 dark:bg-transparent dark:text-white dark:placeholder:text-neutral-400"
          /> : <input
            type="text"
            name="transfer-to"
            placeholder="abcd..."
            autoComplete="off"
            value={to}
            className="w-full text-ellipsis overflow-hidden text-xs relative rounded-lg border bg-white pl-4 pr-8 py-2  text-sm text-black placeholder:text-neutral-500 dark:border-neutral-800 dark:bg-transparent dark:text-white dark:placeholder:text-neutral-400"
          />}
          <div className="absolute right-0 top-0 mr-3 flex h-full items-center">
            To
          </div>
        </div>
        <div className="relative mt-2 flex place-items-center before:absolute before:h-[50px] before:w-[660px] before:-translate-x-1/2 before:rounded-full">
          {preSetAmount ?
            <input
              type="number"
              name="amount"
              autoComplete="off"
              placeholder="not all your money."
              value={parseInt(preSetAmount)}
              disabled={true}
              className="w-max-[560px] relative w-full lg:w-160 xl:w-full rounded-lg border bg-white px-4 py-2 text-sm text-black placeholder:text-neutral-500 dark:border-neutral-800 dark:bg-transparent dark:text-white dark:placeholder:text-neutral-400"
            /> :
            <input
              type="number"
              name="amount"
              autoComplete="off"
              placeholder="not all your money."
              value={amount as number}
              onChange={handleAmountChange}
              className="w-max-[560px] relative w-full lg:w-160 xl:w-full rounded-lg border bg-white px-4 py-2 text-sm text-black placeholder:text-neutral-500 dark:border-neutral-800 dark:bg-transparent dark:text-white dark:placeholder:text-neutral-400"
            />
          }
          <div className="absolute right-0 top-0 mr-3 flex h-full items-center">
            Amount
          </div>
        </div>
        <button onClick={handleSend} disabled={!amountOk() || !toOk()} className="mt-6 ml-auto mr-auto w-[100px] bg-transparent hover:bg-gray-900 text-white font-semibold py-2 px-4 border border-gray-700 rounded shadow disabled:bg-gray-900 disabled:cursor-not-allowed">
          Confirm
        </button>
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
