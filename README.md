# A core Lightning LSPS implementation

The Lightning Service Provider Specification (LSP-spec) standardizes how LSP-clients and LSP-servers interact. 
LSP-clients can purchase channels from LSP-servers to ensure they are well-connected to the network and can receive payments.

This repository aims to provide
- a high quality client implementation for Core Lightning (`plugins/lsps0-client`)
- a library that can be used to implement your own LSP-client or LSP-server (`libs/lsps-primitives`)

The LSP-server should not be used in production. It serves as a testing-ground for the LSP-client and has been developped for that purpose.
However, I welcome any contribution that helps to make it production ready.

## Features

| Spec     | Feature                    | Client | Server |
|----------|----------------------------|--------|--------|
| LSPS0    | Transport layer            | Yes    | Yes    |
| LSPS1    | Purchasing channels        | No     | No     |
| LSPS2    | JIT-channel                | No     | No     |

