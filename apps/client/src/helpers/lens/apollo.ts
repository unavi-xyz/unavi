import {
  ApolloClient,
  ApolloLink,
  HttpLink,
  InMemoryCache,
} from "@apollo/client";
import { LOCAL_STORAGE } from "./constants";

const httpLink = new HttpLink({ uri: "https://api-mumbai.lens.dev/" });

const authLink = new ApolloLink((operation, forward) => {
  const accessToken = localStorage.getItem(LOCAL_STORAGE.ACCESS_TOKEN);

  operation.setContext({
    headers: {
      "x-access-token": accessToken ? `Bearer ${accessToken}` : "",
    },
  });

  return forward(operation);
});

export const apolloClient = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});
