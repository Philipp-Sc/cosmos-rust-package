# cosmos-rust-package
Package used by https://t.me/cosmos_governance_briefings_bot.

- An API to query and broadcast transactions via gRPC.
- Makes direct use of [cosmos-rust](https://github.com/cosmos/cosmos-rust) (cosmos‑sdk‑proto, cosmrs) and [osmosis-rust](https://github.com/osmosis-labs/osmosis-rust) (osmosis-std).

## Developer notes

#### cosmos-rust-package/src/api/core/cosmos/channels/mod.rs 

- load blockchains defined in `tmp/supported_blockchains.json` via [chain-registry](https://github.com/cosmos/chain-registry) repository and test connection.
- get channel to gRPC node for supported blockchain. 

#### cosmos-rust-package/src/api/core/cosmos/keys/
- `key management` from [cosm-rust-script](https://github.com/CyberHoward/cosm-rust-script)

#### cosmos-rust-package/src/api/core/cosmos/query/mod.rs 
- `cosmos query interface, returns a proto type`    
- Includes:
  - `query/auth`
  - `query/gov`
  - `query/staking`
  - (feel free to add a PR or open an Issue for queries you need that are not yet added here)

#### cosmos-rust-package/src/api/core/osmosis/query/mod.rs 
- `osmosis query interface, returns a proto type`

#### cosmos-rust-package/src/api/custom/query/mod.rs 
- `custom query interface, extended to return a custom type, easier to work with`
- Includes: 
  - `query/gov`
  - `query/staking`
  - (feel free to add a PR or open an Issue for queries you need that are not yet added here)


#### cosmos-rust-package/src/api/custom/types/mod.rs 
- `custom query interface, custom return types, easier to work with`
- `adds serialize/deserialize`
- `adds useful helper methods to work with the inner proto type`
- Includes: 
  - `query/gov`
  - `query/staking`
  - (feel free to add a PR or open an Issue for queries you need that are not yet added here)

## Similar Projects

- <a href="https://github.com/PeggyJV/ocular">PeggyJV/ocular</a>
- <a href="https://github.com/CyberHoward/cosm-rust-script">CyberHoward/cosm-rust-script</a> 
