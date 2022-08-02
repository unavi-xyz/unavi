import { useContext, useEffect, useState } from "react";

import { EthersContext } from "@wired-xr/ethers";
import { LocalStorage, SessionStorage } from "@wired-xr/lens";

import { useJWTStore } from "./store";
import { trpc } from "./trpc";

export function useAuthenticate() {
  const [authenticated, setAuthenticated] = useState(false);

  const { address, signer } = useContext(EthersContext);

  const { mutateAsync: login } = trpc.useMutation("login");
  const { data, refetch, isLoading } = trpc.useQuery(["authenticated"], {
    enabled: false,
    retry: false,
  });

  useEffect(() => {
    async function authenticate() {
      if (isLoading) return;

      if (!address || !signer) {
        setAuthenticated(false);
        useJWTStore.setState({ token: "" });
        return;
      }

      const tokenKey = `${LocalStorage.DatabaseToken}_${address}`;
      const prevToken = localStorage.getItem(tokenKey);

      if (data === true) {
        setAuthenticated(true);
        const token = useJWTStore.getState().token;
        localStorage.setItem(tokenKey, token);
        return;
      }

      if (prevToken) {
        useJWTStore.setState({ token: prevToken });
        localStorage.removeItem(tokenKey);
        refetch();
        return;
      }

      // Generate a new token
      const expirationDate = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000);
      const expiration = expirationDate.getTime();

      // Prompt user to sign a message to verify their identity
      const line1 = "The Wired Login";
      const line2 = `Expiration: ${expirationDate.toLocaleDateString()}`;
      const line3 = `Server: ${window.location.host}`;
      const signature = await signer.signMessage(`${line1}\n${line2}\n${line3}`);

      const { token } = await login({
        address,
        signature,
        expiration,
      });

      //save JWT token
      localStorage.setItem(tokenKey, token);
      sessionStorage.setItem(SessionStorage.ActiveDatabaseToken, token);

      // Update the store
      useJWTStore.setState({ token });

      setAuthenticated(true);
    }

    try {
      authenticate();
    } catch (e) {
      console.error(e);
      setAuthenticated(false);
      useJWTStore.setState({ token: "" });
    }
  }, [address, signer, data, isLoading, login, refetch]);

  return { authenticated };
}
