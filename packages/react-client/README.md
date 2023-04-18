# React Client

React components and hooks for running a Wired client.

## Installation

```bash
yarn install @unavi/react-client
```

## Usage

- Requires the website to be [Cross Origin Isolated](https://web.dev/coop-coep/).
- Requires [draco_decoder.js](https://github.com/google/draco/tree/master/javascript/example) to be loaded on the page.

The `Client` component is a self-contained client for UNAVI. It manages the connection to the host and renders the scene. Components can be passed in as children, and can access internal state using the `ClientContext` (for example, if you want to send messages to the host using the WebSocket connection).

```jsx
import { Client } from "@unavi/react-client";

export default function App() {
  return (
    <Client
      uri="https://path.to/space.glb"
      metadata={{...}}
      host="wss://host.unavi.xyz"
      skybox="/skybox.jpg"
      defaultAvatar="/default-avatar.vrm"
      animations="/animations"
    />
  );
}
```
