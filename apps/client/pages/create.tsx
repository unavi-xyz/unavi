import { useContext, useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";

import { LensContext } from "@wired-xr/lens";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import { LoginContext } from "../src/trpc/LoginProvider";
import { trpc } from "../src/trpc/trpc";
import CreateScenePage from "../src/ui/CreateScenePage";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";
import Card from "../src/ui/base/Card";
import Dialog from "../src/ui/base/Dialog";

export default function Create() {
  const [openCreateSpace, setOpenCreateSpace] = useState(false);

  const { data, status, refetch } = trpc.useQuery(["projects"], {
    enabled: false,
  });

  const utils = trpc.useContext();
  const { handle } = useContext(LensContext);
  const { authenticated } = useContext(LoginContext);

  useEffect(() => {
    if (authenticated) {
      utils.invalidateQueries(["projects"]);
      refetch();
    }
  }, [handle, utils, authenticated, refetch]);

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
              <Button variant="outlined" squared="small" onClick={() => setOpenCreateSpace(true)}>
                <MdAdd className="text-lg" />
              </Button>
            </div>
          </div>

          {status === "error" && <div className="text-center">Error</div>}
          {status === "loading" && <div className="text-center">Loading...</div>}

          <div className="grid grid-cols-3 gap-2">
            {data?.map(({ id }) => (
              <Card key={id} />
            ))}
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
