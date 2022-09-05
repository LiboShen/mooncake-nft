import React from "react";
import { topRank } from "./near-api";

export default function KarmaboardPage() {
  const [rank, setRank] = React.useState([]);

  React.useEffect(() => {
    topRank().then((r) => setRank((_) => r));
  }, []);

  return (
    <div>
      <div className="text-2xl text-center mb-2">The Karmaboard</div>
      <div className="text-sm text-center mb-8 text-gray-400">
        Sending your friends Mooncake earns karma for you.
      </div>
      <div>
        {rank.reverse().map(([karma, accountId]) => (
          <div
            key={accountId}
            className="flex flex-auto flex-row justify-between space-x-4 max-w-xl mb-4 mx-auto"
          >
            <div>{accountId}</div> <div>{karma}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
