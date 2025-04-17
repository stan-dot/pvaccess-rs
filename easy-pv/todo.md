

# todo

using the outlining strategy
- [ ] make 3 new packages with hello world
    - [x] make a library for the datatypes
    - [x] binary for client
    - [x] binary for server
    - [x] library for server logic
    - [x] import the server library into the server
    - [x] add a mod file

- [ ] define the state
  - [x] add the config crate
  - [ ] set up the features, dictionary inside the main server state
    - [ ] confirm that the features work
  - [x] make a full function to extract the settings - no parameters one
  - [x] use oneshot for signal termination
  - [x] read out from the settings
  - [ ] work out the connection caching
  - [x] divide the settings sensibly`
  - [ ] add try into instead of from_bytes, it's more idiomatic
- [ ] pass the endianness flag into the handling of the frame
- [ ] server parts
  - [x] read the config
  - [ ] start the udp task
  - [ ] start the tcp task and accept for various headers
  - [ ] add the terminate signal
  - [ ] construct the terminate logic
  - [ ] construct connection validation request
  - [ ] connect the fieldesc
  - [ ] each handler will get typed param and return also type - into bytes is separate
- [ ] minimal working client and server for
  - [ ] udp discovery
  - [ ] echo and persistent connection
  - [ ] websocket show status
- [ ] parse the header
- [ ] start the udp beacon

