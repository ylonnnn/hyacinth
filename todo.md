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
- [x] implement primitive type casting
- [ ] implement pointers and corresponding casts and type checking/inference
- [x] implement interface type extensions

### Fix

- [x] fix MIR lowering issues from lowering anonymous functions
- [x] fix an issue regarding paths that have tailing segments with generics
- [x] fix an issue with usage of associated items within local extensions
- [x] fix diagnostic CLI reporting emphasis ordering
- [x] fix diagnostic CLI reporting emphasis pointer and line alignment
- [x] fix an issue with the phases when dealing with definitions of duplicates
- [ ] fix types of anonymous functions being considered as unresolved types
- [ ] fix unresolved type analysis issues: non-greedy error finding, inaccurate node pointers
- [x] fix invalid inference errors
- [x] fix an issue with the method call expression inference failing to distinguish unrelated matches
- [x] fix an issue where unrecognized items/symbols cause cascading function call errors(specifically, illegal invocation error)
- [x] fix an issue where items within extensions have their own invalid states (e.g. a function being of type Infer(InferKind::Any) if the extensions are not used)
- [ ] fix an issue where recursion causes type computation cycle errors
- [x] fix an issue where arguments of method calls are not checked
- [x] fix an issue where the types of anonymous functions are not properly initially unified
- [x] fix an issue with the expression branches of `if` expressions by adding some sort of disambiguator

### Update/Refactor

- [x] separate extension look-up based on whether the target is nominal (consists of their own definitions/is defined), or structural (a basic layer for efficiency which does not have a constant time look-up) which simply compares the structure of the target type
- [x] clean-up `hycc_diagnostic` implementation
- [ ] disallow usage of non-local generic parameters
- [x] improve diagnostic reporting
- [x] remodel the phases to improve experience and be more intuitive
- [ ] proceed with attaching types regardless of the path argument invalidity
- [x] move invalid inference errors to the type inference phase for better diagnostics
- [x] allow expressions to be the branches of `if` expressions for flexibility and convenience
- [x] improve the return type of `TyInferer::check()`
- [x] allow expressions to be the body of an anonymous function and be used as the implicit return value for flexibility and convenience
- [x] allow terminator requirements to accept `}` as a valid alternative for terminators indicating that the block is enclosed meaning that the statement will not further proceed
- [ ] update the accessibility modifiers to allow for either current petal relativity or target petal relativity as the source
- [ ] path expressions should provide a helpful diagnostic when the last segment is a petal
- [x] rewrite the inferface items

### Test

- [ ] check behavior of having an error in the `collection` phase and in the `resolution` phase at the same time.
- [ ] add unit tests which cover all features
