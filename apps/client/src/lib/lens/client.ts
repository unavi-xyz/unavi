import { createClient } from "urql";

import { useEthersStore } from "../ethers/store";
import { API_URL, LOCAL_STORAGE } from "./constants";

function getAccessToken() {
  const address = useEthersStore.getState().address;
  if (!address) return;

  const accessToken = localStorage.getItem(
    `${address}${LOCAL_STORAGE.ACCESS_TOKEN}`
  );

  return accessToken;
}

export const lensClient = createClient({
  url: API_URL,
  fetchOptions: () => {
    const token = getAccessToken();

    return {
      headers: { authorization: token ? `Bearer ${token}` : "" },
    };
  },
});
