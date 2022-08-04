import { useSession } from "next-auth/react";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";
import { useAccount } from "wagmi";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import { trpc } from "../src/login/trpc";
import CreateScenePage from "../src/ui/CreateScenePage";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";
import Card from "../src/ui/base/Card";
import Dialog from "../src/ui/base/Dialog";

export default function Create() {
  const [openCreateSpace, setOpenCreateSpace] = useState(false);

  const { status: authState } = useSession();
  const { status: accountStatus } = useAccount();
  const utils = trpc.useContext();

  const { data, status, refetch } = trpc.useQuery(["projects"], {
    enabled: authState === "authenticated",
    retry: false,
  });

  useEffect(() => {
    if (authState !== "authenticated" || accountStatus !== "connected") return;
    utils.invalidateQueries(["projects"]);
    refetch();
  }, [utils, refetch, authState, accountStatus]);

  return (
    <>
      <MetaTags title="Create" />

      <Dialog open={openCreateSpace} onClose={() => setOpenCreateSpace(false)}>
        <CreateScenePage />
      </Dialog>

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">Projects</div>
            <div>
              {authState === "authenticated" && (
                <Button variant="outlined" squared="small" onClick={() => setOpenCreateSpace(true)}>
                  <MdAdd className="text-lg" />
                </Button>
              )}
            </div>
          </div>

          {status === "error" && <div className="text-center">Error</div>}
          {status === "loading" && <div className="text-center">Loading...</div>}

          <div className="grid grid-cols-3 gap-2">
            {authState === "authenticated" && data?.map(({ id }) => <Card key={id} />)}
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
