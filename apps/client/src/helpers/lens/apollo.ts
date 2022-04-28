import {
  ApolloClient,
  ApolloLink,
  HttpLink,
  InMemoryCache,
} from "@apollo/client";

import { useEthersStore } from "../ethers/store";
import { LOCAL_STORAGE } from "./constants";

const httpLink = new HttpLink({ uri: "https://api-mumbai.lens.dev/" });

const authLink = new ApolloLink((operation, forward) => {
  const address = useEthersStore.getState().address;

  if (address) {
    const accessToken = localStorage.getItem(
      `${address}${LOCAL_STORAGE.ACCESS_TOKEN}`
    );

    operation.setContext({
      headers: {
        "x-access-token": accessToken ? `Bearer ${accessToken}` : "",
      },
    });
  }

  return forward(operation);
});

export const apolloClient = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});
