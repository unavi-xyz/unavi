import { ethers } from "ethers";
import { LensPeriphery__factory } from "../../../contracts";

export const LENS_PERIPHERY_ADDRESS =
  "0x702C22BFCD705c42B46Df8512b51311a2B5e6036";
const LENS_PERIPHERY_ABI = require("../../../contracts/abis/lensPeriphery.abi");

// export const lensPeriphery = new ethers.Contract(
//   LENS_PERIPHERY_ADDRESS,
//   LENS_PERIPHERY_ABI
// );

// export const lensPeriphery = LensPeriphery__factory.connect(LENS_PERIPHERY_ADDRESS)
