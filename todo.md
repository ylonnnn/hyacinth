# To-Do

- [x] implement generic functions
- [x] implement generic structs
- [ ] protocol (`proto`) declarations
- [ ] expression mutability
- [ ] fix `MirDef`s of anonymous functions
- [x] fix an issue regarding paths that have tailing segments with generics
- [ ] disallow usage of non-local generic parameters
- [x] fix an issue with usage of associated items within local extensions
- [x] define `Self` per impl scope
- [x] separate extension look-up based on whether the target is nominal (consists of their own definitions/is defined), or structural (a basic layer for efficiency which does not have a constant time look-up) which simply compares the structure of the target type
- [ ] implement some sort of merging step for separate extensions to detect duplications
- [ ] clean-up `hycc_diagnostic` implementation
