export const faunadb = require("faunadb");

export const client = new faunadb.Client({
  secret: process.env.FAUNA_KEY,
  domain: "db.us.fauna.com",
});
