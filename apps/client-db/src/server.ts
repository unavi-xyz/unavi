import * as trpcExpress from "@trpc/server/adapters/express/dist/trpc-server-adapters-express.cjs";
import cors from "cors";
import dotenv from "dotenv";
import express from "express";

import { createContext } from "./context";
import { router } from "./router";

dotenv.config();

start();

async function start() {
  //create express server
  const app = express();

  //enable cors
  app.use(
    cors({
      origin: "*",
    })
  );

  app.use(
    "/trpc",
    trpcExpress.createExpressMiddleware({
      router,
      createContext,
    })
  );

  app.get("/ping", (_, res) => {
    res.send("pong");
  });

  //start http server
  const port = 3000;
  app.listen(port);
  console.log(`Server started on port ${port}`);
}
