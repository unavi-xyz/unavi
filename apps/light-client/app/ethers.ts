import { ethers } from "ethers";

export const ethersProvider = new ethers.providers.JsonRpcProvider(process.env.ETH_PROVIDER);
