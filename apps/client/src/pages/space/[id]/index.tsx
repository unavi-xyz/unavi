import { NextPageContext } from "next";

import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import {
  getSpaceLayoutProps,
  SpaceLayoutProps,
} from "../../../home/layouts/SpaceLayout/getSpaceLayoutProps";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=30");

  const props = await getSpaceLayoutProps(query.id as string);

  return {
    props,
  };
}

export default function Space(props: SpaceLayoutProps) {
  return (
    <SpaceLayout {...props}>
      <div className="space-y-2">
        <div className="text-2xl font-bold">Description</div>
        <div className="text-lg text-outline">
          {props.publication?.metadata.description}
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;
