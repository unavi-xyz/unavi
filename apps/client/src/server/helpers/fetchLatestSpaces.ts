import { Space__factory, SPACE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";
import { validateSpace, ValidResponse } from "./validateSpace";

export async function fetchLatestSpaces(limit: number, owner?: string) {
  const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  const count = (await spaceContract.count()).toNumber();

  const spaces: ValidResponse[] = [];
  const length = Math.min(limit, count);
  let nextSpaceId = count - 1;

  const fetchSpace = async () => {
    if (nextSpaceId === 0 || spaces.length === length) return;

    const valid = await validateSpace(nextSpaceId--, owner);

    if (valid) spaces.push(valid);
    else await fetchSpace();
  };

  await Promise.all(Array.from({ length }).map(fetchSpace));

  return spaces.sort((a, b) => b.id - a.id);
}
