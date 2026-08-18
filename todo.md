# To-Do

- [x] implement generic functions
- [x] implement generic structs
- [ ] protocol (`proto`) declarations
- [ ] expression mutability
- [x] fix MIR lowering issues from lowering anonymous functions
- [x] fix an issue regarding paths that have tailing segments with generics
- [ ] disallow usage of non-local generic parameters
- [x] fix an issue with usage of associated items within local extensions
- [x] define `Self` per impl scope
- [x] separate extension look-up based on whether the target is nominal (consists of their own definitions/is defined), or structural (a basic layer for efficiency which does not have a constant time look-up) which simply compares the structure of the target type
- [ ] implement some sort of collision-check for separate extensions
- [x] clean-up `hycc_diagnostic` implementation
- [x] improve diagnostic reporting
- [ ] fix diagnostic CLI reporting emphasis ordering
- [ ] remodel the phases to improve experience and be more intuitive
- [ ] check behavior of having an error in the `collection` phase and in the `resolution` phase at the same time.
- [ ] proceed with attaching types regardless of the path argument invalidity
- [ ] fix an issue with the phases when dealing with definitions of duplicates
