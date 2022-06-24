import { Create } from "faunadb";
import { NextApiHandler } from "next";

import { client } from "../../../../src/lib/faunadb/client";
import { SpaceViewEvent } from "../../../../src/lib/faunadb/types";

const handler: NextApiHandler = async (req, res) => {
  const host = req.headers.host;
  const referer = req.headers.referer;
  if (!host || !referer) return res.status(400).json({ error: "Bad Request" });

  //requires the request to be from the same domain
  const refererHost = new URL(referer).host;
  if (host !== refererHost) return res.status(403).json({ error: "Forbidden" });

  const viewEvent: SpaceViewEvent = {
    spaceId: req.query.id as string,
  };

  //create the view event
  await client.query(Create("Space-View-Events", { data: viewEvent }));

  res.status(200);
};

export default handler;
