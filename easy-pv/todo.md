

# todo

using the outlining strategy
- [ ] make 3 new packages with hello world
    - [ ] make a library for the datatypes
    - [ ] binary for client
    - [ ] binary for server
    - [ ] library for server logic
    - [ ] import the server library into the server
    - [ ] add a mod file
- [ ] define the state
  - [ ] add the config crate
  - [ ] set up the features, dictionary inside the main server state
  - [ ] make a full function to extract the settings - no parameters one
  - [ ] use oneshot for signal termination
  - [ ] read out from the settings
  - [ ] work out the connection caching
  - [ ] divide the settings sensibly`
- [ ] pass the endianness flag into the handling of the frame
- [ ] server parts
  - [ ] read the config
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

