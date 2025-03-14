
# todos for this project

- [x] UDP liveness
- [x] define basic test-schemas (no business logic)
- [x] working server client for arbitrary schemas
- [x] python end to end - investigating maturin - not yet really, might not need the python part, maturin investigated
- [x] a worker pool implementation - instead we use autoscaling - documented inside adrs


## next basic step

- [ ] make sure server and client can connect and send confirmation message
- [ ] check echo message
- [ ] check multi client connections


## big crucial step - stateless service
- [ ] test docker-compose for https://github.com/dragonflydb/dragonfly - check out Garry's stuff for this maybe the machine would work after the rebuild
- [ ] make channels setup with msgpack and redis - and design for arbitrary protocol


## further
- [ ] send UUIDs - initialized locally - asked in k8s pod - use HOSTNAME for the pod if right
maybe
- [ ] define the pvaccess protocol in a separate package and feature

- [ ] implement the pvaccess messages
- [ ] implement the pvaccess channel logic

- [ ] Protocol Messages 
    - [ ] Message Header 
- [ ] Application Messages 
    - [ ] Beacon (0x00) 
    - [ ] Connection validation (0x01
    - [ ] Echo (0x02) 
    - [ ] Search request (0x03) 
    - [ ] Search response (0x04) 
    - [ ] Create channel (0x07) 
    - [ ] Destroy channel (0x08) 
    - [ ] Channel get (0x0A) 
    - [ ] Channel put (0x0B) 
    - [ ] Channel put-get (0x0C) 
    - [ ] Channel monitor (0x0D) 
    - [ ] Channel array (0x0E) 
    - [ ] Destroy request (0xF) 
    - [ ] Channel process (0x10) 
    - [ ] Get channel type introspection data (0x11) 
    - [ ] Message (0x12) 
    - [ ] Channel RPC (0x14) 
    - [ ] Cancel request (0x15) 
- [ ] Control Messages 
    - [ ] Mark Total Byte Sent (0x00) 
    - [ ] Acknowledge Total Bytes Received (0x01) 
    - [ ] Set byte order (0x02) 
    - [ ] Echo request (0x03) 
    - [ ] Echo response (0x04) 


## after the grafana - kafka discussion

- [ ] TUI visualization
- [ ] REACT visualization iff websockets