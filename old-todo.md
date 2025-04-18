
# todos for this project

- [x] UDP liveness
- [x] define basic test-schemas (no business logic)
- [x] working server client for arbitrary schemas
- [x] python end to end - investigating maturin - not yet really, might not need the python part, maturin investigated
- [x] a worker pool implementation - instead we use autoscaling - documented inside adrs

## next basic step

- [x] make sure server and client can connect and send confirmation message
- [x] check echo message
- [x] send UUIDs - initialized locally - use HOSTNAME for the pod if right
- [x] check multi client connections - yeah should work not necessarily from one host for TCP

## DX

- [x] add examples dir <https://doc.rust-lang.org/cargo/reference/cargo-targets.html> - just binaries in examples is fair enough, idk why it doesn't work but I won't let this stop me
- [x] consider cargo make for running tests <https://github.com/sagiegurari/cargo-make> - that is just overkill for now maybe would be useful later.
- [x] define the pvaccess protocol in a separate package and feature, so msgpack as one, and pvaccess as another
- [x] make channels setup with msgpack - and design for arbitrary protocol - yeah add the cfg feature setup for many protocols
- [x] add tests for various scenarios and regression testing - later after the start

## conjectural and not essential - putting on a backburner for a moment

- [ ] test docker-compose for <https://github.com/dragonflydb/dragonfly>
- [ ] TUI visualization - could be just a redis client tbh - that's the smart way about it - but wait maybe redis is on the network and worse latency - RAM is definitely faster
- [ ] make msgpack - redis setup - make it all work - might not be needed, as not much state is stored really on the backend

## work package
- [ ] add client sessions
- [ ] work through the encoding now - just throw error on unknown types - move the todo comments into code comments
- [ ] REACT visualization iff websockets - that is strivial if the server is stateless with redis - <https://uibakery.io/blog/redis-gui-tools> - not doing that after all

## pvaccess stateless start

- [x] Protocol Messages - start
  - [x] Message Header - easy just 4 bytes and an int - 8 in total tbh

- [x] easy messages to start with
  - [x] Beacon (0x00)
  - [x] Connection validation (0x01)
  - [x] Echo (0x02) - with arbitrary bytes
  - [x] Message (0x12)  - human readable into the client - start with this one

  - [x] todo make the socket address everywhere just a String, just wrap when sending

- [x] application messages - UDP discovery
  - [x] Search request (0x03)
  - [x] Search response (0x04)

## channel CRUD stuff - from Application Messages - by here need redis - let's just write in memory for concept simplicity and make the redis-backed version later

- [ ] Create channel (0x07) - must make an equivalent to json-schema. - maybe that is feature creep
- [ ] Destroy channel (0x08)
- [ ] Channel get (0x0A)
- [ ] Channel put (0x0B)
- [ ] Channel put-get (0x0C) - not for now really
- [ ] Channel monitor (0x0D)
- [ ] Get channel type introspection data (0x11) - basic channel meta read

## feels not needed atm

### cherry on the top

- [ ] add tracing - would be easy after setting up the websockets

- [ ] Control Messages  - hard part
  - [ ] Mark Total Byte Sent (0x00) - idk if necessary
  - [ ] Acknowledge Total Bytes Received (0x01)
  - [ ] Set byte order (0x02) - sent first after the connection is established

### beyond MVP

- [ ] Channel array (0x0E)  - some multiple values setup, unclear

- [ ] Channel RPC (0x14)   - init and other requests
- [ ] Channel process (0x10) - execute code associated with the channel?? weird, similar to RPC
- [ ] Application Messages  - miscellanea - that is in regards to rpc
  - [ ] Destroy request (0xF) - what is the difference from cancel request? request instance not pending request. they have the same signature though
  - [ ] Cancel request (0x15)  - just the same almost

## not sure

- [ ] Control Messages  - easy part- not sure what that means
  - [ ] Echo request (0x03) - diagnostic
  - [ ] Echo response (0x04) - diagnostic response
