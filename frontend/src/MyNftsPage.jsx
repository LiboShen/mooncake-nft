import React from "react";
import { signInWithNearWallet, myTokens } from "./near-api";
import { nearConfig } from "./near-api";

export default function MyNftsPage() {
  let [tokens, setTokens] = React.useState([]);
  React.useEffect(() => {
    myTokens().then((tokens) => {
      console.log(tokens);
      setTokens(tokens);
    });
  }, []);

  return (
    <div>
      <div className="text-3xl mb-4 text-center">My Mooncakes</div>

      <div className="grid gap-4 mb-8 grid-cols-1 grid-rows-3 sm:grid-cols-2 sm:grid-rows-3 lg:grid-cols-3 lg:grid-rows-2">
        {tokens.map(({ metadata: { media: media } }, i) => (
          <img
            key={i}
            className={
              "h-72 w-72 border rounded-md border-white mx-auto" +
              (i >= 3 ? " hidden sm:block" : "")
            }
            src={media}
          />
        ))}
      </div>
      <div className="flex flex-col lg:flex-row space-y-4 lg:space-y-0 justify-between">
        <div className="font-sm mb-8 text-gray-300">
          The NFT image will be algorithmically generated and stored on-chain.
          Each mint will have a unique glitch.
        </div>
      </div>
    </div>
  );
}
