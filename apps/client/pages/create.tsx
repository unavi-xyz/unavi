import Link from "next/link";
import { useState } from "react";
import { MdArrowBackIosNew } from "react-icons/md";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import AvatarUploadPage from "../src/home/lens/AvatarUploadPage";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";
import Dialog from "../src/ui/base/Dialog";

export default function Create() {
  const [openAvatar, setOpenAvatar] = useState(false);

  return (
    <>
      <MetaTags title="Create" />

      <Dialog open={openAvatar} onClose={() => setOpenAvatar(false)}>
        <AvatarUploadPage />
      </Dialog>

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="space-y-2">
            <Link href="/studio">
              <a>
                <Button variant="tonal" fullWidth squared="large">
                  <div className="p-2 flex items-center justify-between text-lg">
                    <div>
                      <div className="flex">Studio</div>
                      <div className="flex font-normal">
                        A visual editor for creating spaces
                      </div>
                    </div>

                    <MdArrowBackIosNew className="rotate-180" />
                  </div>
                </Button>
              </a>
            </Link>

            <Button
              variant="tonal"
              squared="large"
              fullWidth
              onClick={() => setOpenAvatar(true)}
            >
              <div className="p-2 flex items-center justify-between text-lg">
                <div>
                  <div className="flex">Avatars</div>
                  <div className="flex font-normal">
                    Upload a VRM avatar and mint it as an NFT
                  </div>
                </div>

                <MdArrowBackIosNew className="rotate-180" />
              </div>
            </Button>
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
