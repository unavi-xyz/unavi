import { useSession } from "next-auth/react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";
import { useAccount } from "wagmi";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import { trpc } from "../src/login/trpc";
import CreateProjectPage from "../src/ui/CreateProjectPage";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";
import Card from "../src/ui/base/Card";
import Dialog from "../src/ui/base/Dialog";

export default function Create() {
  const [openCreateProject, setOpenCreateProject] = useState(false);

  const { status: authState } = useSession();
  const { status: accountStatus } = useAccount();
  const utils = trpc.useContext();

  const { data, status, refetch } = trpc.useQuery(["projects"], {
    enabled: authState === "authenticated",
  });

  useEffect(() => {
    if (authState !== "authenticated" || accountStatus !== "connected") return;
    utils.invalidateQueries(["projects"]);
    refetch();
  }, [utils, refetch, authState, accountStatus]);

  return (
    <>
      <MetaTags title="Create" />

      <Dialog
        open={openCreateProject}
        onClose={() => setOpenCreateProject(false)}
      >
        <CreateProjectPage />
      </Dialog>

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">Projects</div>
            <div>
              {authState === "authenticated" && (
                <Button
                  variant="outlined"
                  squared="small"
                  onClick={() => setOpenCreateProject(true)}
                >
                  <MdAdd className="text-lg" />
                </Button>
              )}
            </div>
          </div>

          {status === "error" && <div className="text-center">Error</div>}
          {status === "loading" && (
            <div className="text-center">Loading...</div>
          )}

          <div className="grid grid-cols-3 gap-2">
            {authState === "authenticated" &&
              data?.map(({ id, name, image }) => (
                <Link key={id} href={`/project/${id}`}>
                  <div>
                    <Card text={name ?? ""} image={image ?? ""} />
                  </div>
                </Link>
              ))}
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
