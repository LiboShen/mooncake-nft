import React from "react";

export default function App() {
  return (
    <>
      <div className="flex flex-row mb-8 space-x-4 justify-between items-start">
        <div className="flex-1 space-y-2">
          <div className="text-3xl font-serif">Mooncake</div>
          <div className="text-base italic font-serif">noun</div>
          <div className="text-base">
            a Chinese bakery product traditionally eaten during the{" "}
            <a
              href="https://www.google.com/search?q=mid-autumn+festival"
              target="_blank"
              className="underline text-indigo-400"
            >
              Mid-Autumn Festival
            </a>
            . Often been gifted to family and friends to give best wishes.
          </div>
        </div>
        <div className="flex-1 space-y-2">
          <div className="text-3xl font-serif">Mooncake NFT</div>
          <div className="text-base italic font-serif">noun</div>
          <div className="text-base">
            a humble virtual (and on chain) Mooncake. Dairy free. Zero calories.
            Keep you in a good mood when added to your wallet. Also a great gift
            for your (crypto) friends.
          </div>
        </div>
      </div>
      <div className="text-3xl mb-4">
        <a href="/nft">Edition #2022: A Glitch or a Feature?</a>
        <div className="text-sm  mb-8">
          In 2022, the mooncake NFT got some problems, just like the crypto
          market. <br />
          The Hanzi gets distorted when you mint a new mooncake. It's different
          every time.
          <br />
          <i>Maybe it's not a glitch, but a feature?</i>
        </div>
      </div>
      <a href="/nft">
        <div className="grid gap-4 mb-8 grid-cols-1 grid-rows-6 sm:grid-cols-2 sm:grid-rows-3 lg:grid-cols-3 lg:grid-rows-2">
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_0.svg"
            alt=""
          />
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_1.svg"
            alt=""
          />
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_2.svg"
            alt=""
          />
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_3.svg"
            alt=""
          />
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_4.svg"
            alt=""
          />
          <img
            className="h-72 w-72 border rounded-md border-white mx-auto"
            src="/assets/e_5.svg"
            alt=""
          />
        </div>
      </a>
      <div className="text-3xl mb-4">See more past editions</div>
      <div className="text-2xl mb-4">Edition #2021</div>
      <div className="flex flex-row mb-8 space-x-4 justify-between items-start">
        <img
          className="h-72 w-72 border rounded-md border-white mx-auto"
          src="/assets/Hash Nuts.png"
          alt=""
        />
      </div>
    </>
  );
}
