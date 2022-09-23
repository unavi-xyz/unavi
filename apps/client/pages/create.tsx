import Link from "next/link";
import { useSession } from "next-auth/react";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";
import { useAccount } from "wagmi";

import { trpc } from "../src/auth/trpc";
import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import Button from "../src/ui/base/Button";
import Card from "../src/ui/base/Card";
import Dialog from "../src/ui/base/Dialog";
import CreateProjectPage from "../src/ui/CreateProjectPage";
import MetaTags from "../src/ui/MetaTags";

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

  const authenticated = authState === "authenticated";

  return (
    <>
      <MetaTags title="Create" />

      <Dialog
        open={openCreateProject}
        onClose={() => setOpenCreateProject(false)}
      >
        <CreateProjectPage />
      </Dialog>

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">⚒️ Projects</div>
            <div>
              <Button
                variant="outlined"
                rounded="small"
                disabled={!authenticated}
                onClick={() => {
                  if (!authenticated) return;
                  setOpenCreateProject(true);
                }}
              >
                <MdAdd className="text-lg" />
              </Button>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-3">
            {authState === "authenticated" &&
              status === "success" &&
              data.map(({ id, name, image }) => (
                <Link key={id} href={`/project/${id}`} passHref>
                  <div>
                    <Card
                      text={name}
                      image={image}
                      sizes="333px"
                      animateEnter
                    />
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
