
# todos for this project

- [x] UDP liveness
- [x] define basic test-schemas (no business logic)
- [x] working server client for arbitrary schemas
- [x] python end to end - investigating maturin - not yet really, might not need the python part, maturin investigated
- [x] a worker pool implementation - instead we use autoscaling - documented inside adrs


## next basic step

- [x] make sure server and client can connect and send confirmation message
- [ ] check echo message
- [ ] check multi client connections
- [ ] send UUIDs - initialized locally - use HOSTNAME for the pod if right

## DX 

- [ ] add tests for various scenarios and regression testing - later after the start
- [ ] add examples dir https://doc.rust-lang.org/cargo/reference/cargo-targets.html
- [x] consider cargo make for running tests https://github.com/sagiegurari/cargo-make - that is just overkill for now maybe would be useful later.
- [ ] test docker-compose for https://github.com/dragonflydb/dragonfly - check out Garry's stuff for this maybe the machine would work after the rebuild https://containers.dev/guide/dockerfile
- [ ] make channels setup with msgpack and redis - and design for arbitrary protocol

## pvaccess stateless start

- [ ] define the pvaccess protocol in a separate package and feature, so msgpack as one, and pvaccess as another

- [ ] Protocol Messages - start
    - [ ] Message Header - easy just 4 bytes and an int

- [ ] easy messages to start with 
    - [ ] Beacon (0x00) 
    - [ ] Connection validation (0x01)
    - [ ] Echo (0x02) 
    - [ ] Message (0x12)  - human readable into the client - start with this one

- [ ] Control Messages  - easy part
    - [ ] Echo request (0x03) - diagnostic
    - [ ] Echo response (0x04) - diagnostic response

- [ ] application messages - UDP discovery
    - [ ] Search request (0x03) 
    - [ ] Search response (0x04) 


## channel CRUD stuff - from Application Messages - by here need redis

- [ ] Search request (0x03) 
- [ ] Search response (0x04) 
- [ ] Create channel (0x07) - must make an equivalent to json-schema.
- [ ] Destroy channel (0x08) 
- [ ] Channel get (0x0A) 
- [ ] Channel put (0x0B) 
- [ ] Channel put-get (0x0C) 
- [ ] Channel monitor (0x0D) 
- [ ] Get channel type introspection data (0x11) - basic channel meta read


## feels not needed atm

### cherry on the top
- [ ] Application Messages  - miscellanea
    - [ ] Destroy request (0xF) - what is the difference from cancel request? request instance not pending request. they have the same signature though
    - [ ] Cancel request (0x15)  - just the same almost

- [ ] Control Messages  - hard part
    - [ ] Mark Total Byte Sent (0x00) - idk if necessary
    - [ ] Acknowledge Total Bytes Received (0x01) 
    - [ ] Set byte order (0x02) - sent first after the connection is established

### beyond MVP
- [ ] Channel array (0x0E)  - some multiple values setup, idk
- [ ] will need a queue for it, 
    - [ ] Channel RPC (0x14)  
- [ ] Channel process (0x10) - execute code associated with the channel?? weird, similar to RPC

### after the grafana - kafka discussion

- [ ] TUI visualization
- [ ] REACT visualization iff websockets