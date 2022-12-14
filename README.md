# cosmos-rust-package
Package used by the cosmos-rust-interface. Makes direct use of cosmos-rust (cosmos‑sdk‑proto, osmosis-proto, cosmrs). 

## Developer notes

- cosmos-rust-package/src/api/core/cosmos/channels/mod.rs 
`supported blockchains with channels to gRPC nodes`

- cosmos-rust-package/src/api/core/cosmos/keys/
`key management` from [cosm-rust-script](https://github.com/CyberHoward/cosm-rust-script)

- cosmos-rust-package/src/api/core/cosmos/query/mod.rs 
`cosmos query interface, returns a proto type`

- cosmos-rust-package/src/api/core/osmosis/query/mod.rs 
`osmosis query interface, returns a proto type`

- cosmos-rust-package/src/api/custom/mod.rs 
`custom query interface, extended functionality with custom return type, easier to work with`

### Goals

1. Simple API to query and broadcast transactions via gRPC 



## Similar Projects

- <a href="https://github.com/PeggyJV/ocular">PeggyJV/ocular</a>
- <a href="https://github.com/CyberHoward/cosm-rust-script">CyberHoward/cosm-rust-script</a> 
