# React Client

React components and hooks for running a lightweight Wired client.

## Installation

```bash
yarn install @wired-labs/react-client
```

## Usage

- Requires the website to be [Cross Origin Isolated](https://web.dev/coop-coep/).
- Requires draco scripts to be loaded on the page.

```jsx
import { Client } from "@wired-labs/react-client";

export default function App() {
  return (
    <Client
      spaceId={13}
      metadata={{...}}
      avatar="/default-avatar.vrm"
      skybox="/skybox.jpg"
    />
  );
};
```
