---
sidebar_position: 30
sidebar_label: 📜 Contracts
title: Contracts
---

The Wired operates using smart contracts on the Ethereum blockchain. These contracts allow the decentralized storage of user profiles and space data.

## 📜 Deployed Contracts

Contracts are currently deployed to the [Ethereum Sepolia Testnet](https://www.alchemy.com/overviews/sepolia-testnet).

| Contract | Address                                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Profile  | [0x4082Df45C81A6cb80492f506cd985EE8E2aac8b5](https://sepolia.etherscan.io/address/0x4082df45c81a6cb80492f506cd985ee8e2aac8b5) |
| Space    | [0x9BC51fe51720D03C4A8c0bc7efE235b3d492D5f7](https://sepolia.etherscan.io/address/0x9bc51fe51720d03c4a8c0bc7efe235b3d492d5f7) |

## 🪪 Profile Contract

The profile contract allows anyone to create a profile on the blockchain. Profiles are free-to-mint NFTs, and allow users to store arbitrary data about themselves - such as their name, profile picture, and social media links.

Each profile can set a handle, which is a unique string that can be used to identify the profile. Handles are made up of a string, followed by an assigned number ID. For example, `Alice#1234`. Handles can be changed at any time.

## 🌍 Space Contract

The space contract allows anyone to create an NFT representing a space and associate it with a metadata file URI. This contract provides a simple yet powerful way to store information about spaces in a decentralized manner.
