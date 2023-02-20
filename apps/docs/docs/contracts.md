---
sidebar_position: 30
sidebar_label: 📜 Contracts
title: Contracts
---

The Wired operates using smart contracts on the Ethereum blockchain. These contracts allow the decentralized storage of user profiles and space data.

## 📜 Deployed Contracts

Contracts are currently deployed to the **Arbitrum Goerli Testnet**.

| Contract | Address |
|----------|---------|
| Profile | [0xF758c4893667ac69F16134a66F19AED8827B1914](https://goerli.arbiscan.io/address/0xf758c4893667ac69f16134a66f19aed8827b1914) |
| Space | [0xE068Dffa00149408007B627bd092051915bB0736](https://goerli.arbiscan.io/address/0xE068Dffa00149408007B627bd092051915bB0736) |

## 🪪 Profile Contract

The profile contract allows anyone to create a profile on the blockchain. Profiles are free-to-mint NFTs, and allow users to store arbitrary data about themselves - such as their name, profile picture, and social media links.

Each profile can set a handle, which is a unique string that can be used to identify the profile. Handles are made up of a string, followed by an assigned number ID. For example, `Alice#1234`. Handles can be changed at any time.

## 🌍 Space Contract

The space contract allows anyone to create an NFT representing a space and associate it with a metadata file URI. This contract provides a simple yet powerful way to store information about spaces in a decentralized manner.
