# cosmos-rust-package

- An API to query and broadcast transactions via gRPC
- Makes direct use of [cosmos-rust](https://github.com/cosmos/cosmos-rust) (cosmos‑sdk‑proto, cosmrs) and [osmosis-rust](https://github.com/osmosis-labs/osmosis-rust) (osmosis-std)
- Developed for Cosmos Governance Notifications: https://t.me/cosmos_governance_briefings_bot      



## Developer notes

#### api/core/cosmos/channels/mod.rs 

- `load blockchains defined in 'tmp/supported_blockchains.json' via the chain-registry repository and test connection`
- `get channel to gRPC node for supported blockchain`

#### api/core/cosmos/keys/mod.rs 
- `key management` from [cosm-rust-script](https://github.com/CyberHoward/cosm-rust-script)

#### api/core/cosmos/query/mod.rs 
- `cosmos query interface, returns a proto type`    

#### api/core/osmosis/query/mod.rs 
- `osmosis query interface, returns a proto type`

#### api/custom/query/mod.rs 
- `custom queries, extended to return custom types`

#### api/custom/types/mod.rs 
- `custom types, easier to work with`
- `adds serialize/deserialize`
- `adds useful helper methods to work with the inner proto type`

### Implemented Modules    
- `query/auth`
- `query/gov`
- `query/staking`
- `(feel free to add a PR or open an Issue for queries you need that are not yet added here)`


## Similar Projects

- <a href="https://github.com/PeggyJV/ocular">PeggyJV/ocular</a>
- <a href="https://github.com/CyberHoward/cosm-rust-script">CyberHoward/cosm-rust-script</a> 
