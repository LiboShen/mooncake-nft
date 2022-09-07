import React from "react";
import { signInWithNearWallet, nftMint2022 } from "./near-api";
import facaiUrl from "/assets/facai.svg";
import { nearConfig } from "./near-api";
import { glitchFacaiUrls } from "./NftImages";

export default function NftPage() {
  let [imageSrc, setImageSrc] = React.useState(facaiUrl);
  let [txHash, setTxHash] = React.useState("");
  React.useEffect(() => {
    let search = window.location.search;
    if (search.startsWith("?transactionHashes=")) {
      setTxHash((_) => search.split("=")[1]);
    }
  }, []);

  return (
    <div>
      {txHash != "" ? (
        <div className="p-2 bg-green-600 rounded-md mb-8 text-center">
          🎉 The{" "}
          <a
            className="underline"
            target="_blank"
            href={nearConfig.explorerUrl + "/transactions/" + txHash}
          >
            transaction
          </a>{" "}
          is sent. The NFT should show up in your{" "}
          <a
            className="underline"
            target="_blank"
            href={nearConfig.walletUrl + "?tab=collectibles"}
          >
            wallet
          </a>{" "}
          soon! 🥮
        </div>
      ) : null}
      <div className="text-3xl mb-4">Edition #2022</div>
      <div className="flex flex-col lg:flex-row space-y-4 lg:space-y-0 justify-between">
        <div className="flex-1 mx-auto">
          <img
            onMouseEnter={(_) => {
              let i = Math.floor(Math.random() * 10);
              setImageSrc((_) => glitchFacaiUrls[i]);
            }}
            onClick={(_) => {
              let i = Math.floor(Math.random() * 10);
              setImageSrc((_) => glitchFacaiUrls[i]);
            }}
            onMouseLeave={(_) => setImageSrc((_) => facaiUrl)}
            className="h-96 w-96  border rounded-md border-white"
            src={imageSrc}
            alt=""
          />
          <div className="text-xl">恭喜发财</div>
        </div>
        <div className="flex-1">
          <div className="text-lg mb-8">
            In 2022, just like the crypto market, the mooncake NFT got some
            serious problems: <br />
            the Hanzi gets distorted when you mint a new mooncake. It's
            different every time.
            <br />
            <i>Maybe it's not a glitch, but a feature?</i>
          </div>
          <div className="text-lg font-medium mb-4">1 NEAR (Ⓝ)</div>
          <div className="mb-8">
            {window.walletConnection.isSignedIn() ? (
              <button
                className="inline-block rounded-md border border-transparent bg-indigo-500 py-2 px-4 text-lg font-medium text-white hover:bg-opacity-75"
                onClick={(_) => nftMint2022(window.accountId)}
              >
                Mint
              </button>
            ) : (
              <button
                className="inline-block rounded-md border border-transparent bg-indigo-500 py-2 px-4 text-lg font-medium text-white hover:bg-opacity-75"
                onClick={signInWithNearWallet}
              >
                Sign in mint
              </button>
            )}
          </div>
          <div className="font-sm mb-8 text-gray-300">
            The NFT image will be algorithmically generated and stored on-chain.
            Each mint will have a unique glitch.
          </div>
        </div>
      </div>
    </div>
  );
}
