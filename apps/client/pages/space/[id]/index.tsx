import { NextPageContext } from "next";

import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../src/home/layouts/SpaceLayout/SpaceLayout";
import {
  SpaceLayoutProps,
  getSpaceLayoutProps,
} from "../../../src/home/layouts/SpaceLayout/getSpaceLayoutProps";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=10");

  const host = res?.req.headers.host;
  const hostUrl = `https://${host}`;

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
