

# todo

using the outlining strategy
- [x] make 3 new packages with hello world
    - [x] make a library for the datatypes
    - [x] binary for client
    - [x] binary for server
    - [x] library for server logic
    - [x] import the server library into the server
    - [x] add a mod file
    - [x] 4th one for datatypes

- [ ] define the state
  - [x] add the config crate
  - [x] read the config
  - [x] set up the features, dictionary inside the main server state
    - [x] confirm that the features work - discard them really, no need for runtime feature addition
  - [x] make a full function to extract the settings - no parameters one
  - [x] use oneshot for signal termination
  - [x] read out from the settings
  - [x] divide the settings sensibly`
- [x] simplify the features parsing really - yeah just doing manually with the KISS principle, match flag following an enum
- [x] add the terminate signal
- [x] construct the terminate logic


## For Wed
- [x] add a simple client - 
- [x] delete the lib features code
- [ ] server parts
  - [x] start the udp task
  - [x] start the tcp task and accept for various headers
  - [ ] construct connection validation request
- [x] add half socket each as parts
- [ ] add frame parsing with tokio_util::codec::FramedRead to the server
- [ ] add echo read-write
  - [ ] to client
  - [ ] to server
- [ ] this should work as a proof of concept
- [ ] add try into instead of from_bytes, it's more idiomatic - do that for the pv_echo copy

## For later
- [ ] work out the connection caching
- [ ] pass the endianness flag into the handling of the frame

  - [ ] connect the fieldesc
  - [ ] each handler will get typed param and return also type - into bytes is separate
- [ ] minimal working client and server for
  - [ ] udp discovery
  - [ ] echo and persistent connection
  - [ ] websocket show status
- [ ] parse the header
- [ ] start the udp beacon

