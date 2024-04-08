# A core Lightning LSPS implementation

> [!WARNING]
> This is code experimental and using it might result in loss of funds
>
> - The LSP-spec isn't final yet and breaking changes might occur.
> - No code in this repository has been formally peer review (yet).

This is a Core Lightning implementation for the Lightning Serverice Provider Specification or [LSPS](https://github.com/BitcoinAndLightningLayerSpecs/lsp) for short.

The LSP-spec standardizes how LSP-clients and LSP-servers interact. 
LSP-clients can purchase channels from LSP-servers to ensure they are well-connected to the network and can receive payments.

This repository contains
- `lsp-primitives`: a rust crate with basic primitives to implement an LSP-code
- `lsps-client`: A Core Lightning plugin that implements an LSP-client
- `lsps-server`: A Core Lightning plugin that impelments an LSP-server

## Current support

| Spec     | Features                   | Client-support | Server-support |
|----------|----------------------------|----------------|----------------|
| LSPS0    | Transport layer            | Yes            | Yes            |
| LSPS1    | Order a channel            | Yes            | Yes            |
| LSPS1    | Pay order with Lightning   | Yes            | Yes            |
| LSPS1    | Pay order onchain          | No             | No             |

This repository aims to provide
- a high quality client implementation for Core Lightning (`plugins/lsps0-client`)
- a library that can be used to implement your own LSP-client or LSP-server (`libs/lsps-primitives`)

The LSP-server should not be used in production. It serves as a testing-ground for the LSP-client and has been developped for that purpose.
However, I welcome any contribution that helps to make it production ready.

# See also

- [Lightning Service Provider Specification](https://github.com/BitcoinAndLightningLayerSpecs/lsp)
