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

## TODO's

- **cln_plugin**
  - [ ] Plugin options can be dynamic. Implement this in the `cln_plugin`-crate
  - [ ] Plugin options should be queryable. 
    Can we provide a `Map<&str, ConfigOption>` instead of a `Vec<ConfigOption>`.
    How to do this without introducing breaking changes?
  - [ ] Plugin options:
    The API is somewhat confusing.
    The semantic meaning of `Option::Value::OptString` vs `Option::Value::String` 
    is different when defining the option from when we are reading the value.
    Can we fix this without introducing breaking changes?
**cln_rpc** 
   - [ ] Cannot be used with external structs (See https://github.com/ElementsProject/lightning/pull/6954) 
   - [ ] Assumes responses arrive in order. I don't think we should care because we usually 
         create a new socket. VERIFY and document
- **lsps0**
  - [ ] Provide a nice type for BOLT-11 invoices
- **lsps1**
  - **server**
    - [ ] Implement onchain payments
    - [x] Implement lightning payments
    - [x] Open the channel after payment
    - [x] Store the channels, orders, etc...
    - [ ] Test refund on channel payment time-out
    - [ ] Perform renames in LSPS1
  - **client**
    - [ ] Use the data-store to store orders and channels
    - [ ] Track violations
      - [ ] Paid but channel was never opened
      - [ ] Paid but channel was never opened and never refunded
      - [ ] Channel closed early
