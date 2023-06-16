import { ethers } from "ethers";

import { env } from "../env.mjs";

export const ethersProvider = new ethers.providers.JsonRpcProvider(
  env.ETH_PROVIDER
);
