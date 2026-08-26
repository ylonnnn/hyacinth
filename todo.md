# To-Do

### Implementation

- [x] implement generic functions
- [x] implement generic structs
- [ ] interface (`intf`) declarations
- [x] implement some sort of collision-check for separate extensions
- [x] implement a guard for multiple associated item match
- [x] implement a basic extension resolution guard to avoid double resolution
- [ ] expression mutability
- [x] define `Self` per impl scope
- [ ] implement an automated instantiator which automatically creates generic arguments (inferred) other than the instantiator for identifier nodes

### Fix

- [x] fix MIR lowering issues from lowering anonymous functions
- [x] fix an issue regarding paths that have tailing segments with generics
- [x] fix an issue with usage of associated items within local extensions
- [ ] fix diagnostic CLI reporting emphasis ordering
- [ ] fix diagnostic CLI reporting emphasis pointer and line alignment
- [ ] fix an issue with the phases when dealing with definitions of duplicates
- [ ] fix types of anonymous functions being considered as unresolved types
- [ ] fix unresolved type analysis issues: non-greedy error finding, inaccurate node pointers
- [x] fix invalid inference errors
- [x] fix an issue with the method call expression inference failing to distinguish unrelated matches
- [ ] fix an issue where unrecognized items/symbols cause cascading function call errors(specifically, illegal invocation error)
- [x] fix an issue where items within extensions have their own invalid states (e.g. a function being of type Infer(InferKind::Any) if the extensions are not used)

### Update/Refactor

- [x] separate extension look-up based on whether the target is nominal (consists of their own definitions/is defined), or structural (a basic layer for efficiency which does not have a constant time look-up) which simply compares the structure of the target type
- [x] clean-up `hycc_diagnostic` implementation
- [ ] disallow usage of non-local generic parameters
- [x] improve diagnostic reporting
- [ ] remodel the phases to improve experience and be more intuitive
- [ ] proceed with attaching types regardless of the path argument invalidity
- [x] move invalid inference errors to the type inference phase for better diagnostics
- [ ] update the consequent and alternate blocks of `if` expressions to be expressions rather than blocks

### Test

- [ ] check behavior of having an error in the `collection` phase and in the `resolution` phase at the same time.
