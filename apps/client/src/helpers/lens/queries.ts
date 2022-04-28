import { gql } from "@apollo/client";

const PROFILE_INFO = gql`
  fragment ProfileInfo on Profile {
    id
    name
    handle
    bio
    location
    website
    twitter
    ownedBy
    metadata
    picture {
      ... on NftImage {
        contractAddress
        tokenId
        uri
        verified
      }
      ... on MediaSet {
        original {
          url
          mimeType
        }
      }
    }
  }
`;

export const GET_PROFILES_BY_ADDRESS = gql`
  query GetProfilesByAddress($address: EthereumAddress!) {
    profiles(request: { ownedBy: [$address] }) {
      items {
        ...ProfileInfo
      }
    }
  }
  ${PROFILE_INFO}
`;

export const GET_DEFAULT_PROFILE = gql`
  query GetDefaultProfile($address: EthereumAddress!) {
    defaultProfile(request: { ethereumAddress: $address }) {
      ...ProfileInfo
    }
  }
  ${PROFILE_INFO}
`;

export const GET_PROFILE_BY_HANDLE = gql`
  query GetProfileByHandle($handle: Handle!) {
    profiles(request: { handles: [$handle], limit: 1 }) {
      items {
        ...ProfileInfo
      }
    }
  }
  ${PROFILE_INFO}
`;

export const GET_CHALLENGE = gql`
  query GetChallenge($address: EthereumAddress!) {
    challenge(request: { address: $address }) {
      text
    }
  }
`;

export const AUTHENTICATE = gql`
  mutation Authenticate($address: EthereumAddress!, $signature: Signature!) {
    authenticate(request: { address: $address, signature: $signature }) {
      accessToken
      refreshToken
    }
  }
`;

export const REFRESH_AUTHENTICATION = gql`
  mutation RefreshAuthentication($refreshToken: Jwt!) {
    refresh(request: { refreshToken: $refreshToken }) {
      accessToken
      refreshToken
    }
  }
`;

export const HAS_TX_HASH_BEEN_INDEXED = gql`
  query HasTxHashBeenIndexed($txHash: TxHash!) {
    hasTxHashBeenIndexed(request: { txHash: $txHash }) {
      ... on TransactionIndexedResult {
        indexed
        txReceipt {
          to
          from
          contractAddress
          transactionIndex
          root
          gasUsed
          logsBloom
          blockHash
          transactionHash
          blockNumber
          confirmations
          cumulativeGasUsed
          effectiveGasPrice
          byzantium
          type
          status
          logs {
            blockNumber
            blockHash
            transactionIndex
            removed
            address
            data
            topics
            transactionHash
            logIndex
          }
        }
        metadataStatus {
          status
          reason
        }
      }
      ... on TransactionError {
        reason
        txReceipt {
          to
          from
          contractAddress
          transactionIndex
          root
          gasUsed
          logsBloom
          blockHash
          transactionHash
          blockNumber
          confirmations
          cumulativeGasUsed
          effectiveGasPrice
          byzantium
          type
          status
          logs {
            blockNumber
            blockHash
            transactionIndex
            removed
            address
            data
            topics
            transactionHash
            logIndex
          }
        }
      }
      __typename
    }
  }
`;

export const CREATE_PROFILE = gql`
  mutation CreateProfile($handle: CreateHandle!) {
    createProfile(
      request: {
        handle: $handle
        profilePictureUri: null
        followNFTURI: null
        followModule: null
      }
    ) {
      ... on RelayerResult {
        txHash
      }
    }
  }
`;

export const SET_PROFILE_METADATA = gql`
  mutation SetProfileMetadata($profileId: ProfileId!, $metadata: Url!) {
    createSetProfileMetadataTypedData(
      request: { profileId: $profileId, metadata: $metadata }
    ) {
      typedData {
        types {
          SetProfileMetadataURIWithSig {
            name
            type
          }
        }
        domain {
          name
          chainId
          version
          verifyingContract
        }
        value {
          nonce
          deadline
          profileId
          metadata
        }
      }
    }
  }
`;

export const SET_PROFILE_IMAGE = gql`
  mutation SetProfileImage($profileId: ProfileId!, $url: Url!) {
    createSetProfileImageURITypedData(
      request: { profileId: $profileId, url: $url }
    ) {
      typedData {
        domain {
          name
          chainId
          version
          verifyingContract
        }
        types {
          SetProfileImageURIWithSig {
            name
            type
          }
        }
        value {
          nonce
          deadline
          imageURI
          profileId
        }
      }
    }
  }
`;

export const SET_DEFAULT_PROFILE = gql`
  mutation SetDefaultProfile($profileId: ProfileId!) {
    createSetDefaultProfileTypedData(request: { profileId: $profileId }) {
      typedData {
        types {
          SetDefaultProfileWithSig {
            name
            type
          }
        }
        domain {
          name
          chainId
          version
          verifyingContract
        }
        value {
          nonce
          deadline
          wallet
          profileId
        }
      }
    }
  }
`;
