import { useEffect, useState } from "react";

import { connectWallet } from "../../../helpers/ethers/connection";
import { useEthersStore } from "../../../helpers/ethers/store";
import { useLensStore } from "../../../helpers/lens/store";
import { apolloClient } from "../../../helpers/lens/apollo";
import { useCloseDialog } from "../../base/Dialog";
import {
  GET_DEFAULT_PROFILE,
  GET_PROFILES_BY_ADDRESS,
} from "../../../helpers/lens/queries";

import {
  GetDefaultProfileQuery,
  GetDefaultProfileQueryVariables,
  GetProfilesByAddressQuery,
  GetProfilesByAddressQueryVariables,
} from "../../../generated/graphql";

import MetamaskFox from "./MetamaskFox";
import CreateProfilePage from "./CreateProfilePage";
import RichButton from "../../base/RichButton";

export default function LoginPage() {
  const address = useEthersStore((state) => state.address);
  const close = useCloseDialog();

  const [showCreatePage, setShowCreatePage] = useState(false);

  useEffect(() => {
    if (!address) return;

    //query address profiles
    apolloClient
      .query<GetProfilesByAddressQuery, GetProfilesByAddressQueryVariables>({
        query: GET_PROFILES_BY_ADDRESS,
        variables: { address },
      })
      .then((profilesRes) => {
        //if the user has no profiles, show the create page
        if (profilesRes.data?.profiles.items.length === 0) {
          setShowCreatePage(true);
          return;
        }

        //query for default profile
        apolloClient
          .query<GetDefaultProfileQuery, GetDefaultProfileQueryVariables>({
            query: GET_DEFAULT_PROFILE,
            variables: { address },
          })
          .then((defaultRes) => {
            if (defaultRes.data?.defaultProfile) {
              //set default profile
              const handle = defaultRes.data.defaultProfile.handle;
              useLensStore.setState({ handle });
            } else {
              //if no default set, choose the first profile
              const handle = profilesRes.data?.profiles.items[0].handle;
              useLensStore.setState({ handle });
            }
            close();
          })
          .catch((err) => {
            console.error(err);
          });
      })
      .catch((err) => {
        console.error(err);
      });
  }, [address, close]);

  if (showCreatePage) return <CreateProfilePage />;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Login</h1>
        <p className="text-lg flex justify-center">
          Choose how you want to connect your wallet
        </p>
      </div>

      <RichButton
        title="Metamask"
        description="A browser extension"
        image={<MetamaskFox width="100px" />}
        onClick={connectWallet}
      />
    </div>
  );
}
