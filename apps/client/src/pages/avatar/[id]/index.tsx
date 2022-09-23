import { NextPageContext } from "next";

import AvatarLayout from "../../../home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import {
  getPublicationProps,
  PublicationProps,
} from "../../../lib/lens/utils/getPublicationProps";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const props = await getPublicationProps(query.id as string);

  return {
    props,
  };
}

export default function Avatar(props: PublicationProps) {
  return (
    <AvatarLayout {...props}>
      <div className="space-y-2">
        <div className="text-2xl font-bold">Description</div>
        <div className="text-outline text-lg">
          {props.publication?.metadata.description}
        </div>
      </div>
    </AvatarLayout>
  );
}

Avatar.getLayout = getNavbarLayout;
