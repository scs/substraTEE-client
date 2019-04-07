# substrate-client
substrate client connecting to substrate node using websocket interface

**WORK IN PROGRESS**

What works:
  1. query genesis_hash
  1. query account nonce
  1. submit and and subscribe extrinsic (transfer from Alice to Bob, 42 units)
  1. watch extrinsic getting finalized
  
TODO:
  1. CLI options (similar to subkey `transfer` syntax)
  1. subscribe and watch events
  1. (maybe?) refactoring to per-use-case state machine architecture
  1. refactoring to ws-library / CLI binary
  1. (nice to have) dynamic API using metadata  

