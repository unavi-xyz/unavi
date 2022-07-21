# @wired-xr/home

A home server for the Wired.

A home server is a trusted, centralized third party that can provide UX benefits for a user. For example, your home server can sign chat messages you send, verifying they were sent by you, but without requiring you to sign them with your wallet. Or your home server can pay gas fees for you, allowing you to send gasless transactions.

Importantly, a user's identity is stored on Ethereum, not on their homeserver. This means that at any time a user can move from one home server to another.

## Features

- Handles user authentication using wallet signing + JWT tokens
- Stores studio projects

## Tech Stack

- Express + tRPC api
- Postgresql database
