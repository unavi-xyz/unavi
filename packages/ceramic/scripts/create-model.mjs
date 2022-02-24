import { writeFile } from "node:fs/promises";
import { CeramicClient } from "@ceramicnetwork/http-client";
import { ModelManager } from "@glazed/devtools";
import { DID } from "dids";
import { Ed25519Provider } from "key-did-provider-ed25519";
import { getResolver } from "key-did-resolver";
import { randomBytes } from "@stablelib/random";

//params
const NAME = "Space";

//init
const schema = await import(`../models/${NAME}/schema.json`);

const DEF_NAME = NAME[0].toLowerCase() + NAME.substring(1);

const seed = randomBytes(32);
const did = new DID({
  provider: new Ed25519Provider(seed),
  resolver: getResolver(),
});
await did.authenticate();

const ceramic = new CeramicClient("https://ceramic-clay.3boxlabs.com");
ceramic.did = did;

const manager = new ModelManager(ceramic);

// Create the schema
const schemaID = await manager.createSchema(NAME, schema);

// Create the definition using the created schema ID
await manager.createDefinition(DEF_NAME, {
  name: NAME,
  description: "",
  schema: manager.getSchemaURL(schemaID),
});

// Publish model to Ceramic node
const model = await manager.toPublished();

// Write published model to JSON file
await writeFile(`models/${NAME}/model.json`, JSON.stringify(model));

console.log(model);
