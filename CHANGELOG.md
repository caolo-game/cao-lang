# Changelog
## v0.1.90


### Bug Fixes

- Fix with_sub_index on LaneCardIndex


### Features

- Current index api for LaneCardIndex

- All sub-cards are now indexable, not just CompositeCards


### Refactor

- Use owned string in CardIndex to simplify lifetimes

- Simplify the card index data model


### Break

- Remove LaneCardIndex::default

## v0.1.89


### Features

- Current index api

## v0.1.88


### Features

- Add composite_card ctor


### Refactor

- Nicer with_sub_index api

## v0.1.87


### Features

- Expand Card Index API


### Refactor

- Use a boxed struct for nested cards to reduce Card size

## v0.1.86


### Features

- Add Module query API to fetch concrete cards

## v0.1.85


### Break

- Revert card splitting, just inline cards...

## v0.1.84


### Refactor

- Remove lifetimes from the public Module

## v0.1.83


### Features

- Decouple cards and lanes, cards are no longer inlined in lanes

## v0.1.82


### Break

- Simplify Module serialization format

## v0.1.81


### Features

- Add submodule imports

- Add super function imports

- Add super module import


### Breaking

- Reserve the `super` keyword

## v0.1.80


### Features

- Add imports


### Break

- Remove the `compute_stack_at_card` function for now

- Remove Lane lookup in parent scopes

## v0.1.79


### Bug Fixes

- Fix undefined behaviour in the Compiler

- Fix leaky objects in RuntimeData

## v0.1.78


### Features

- Add `compute_stack_at_card` to Lanes

- Add recursion limit to compile options


### Refactor

- Rename hashing function

## v0.1.77


### Features

- Add a hashing function to Programs

## v0.1.76


### Bug Fixes

- Fix panic on empty target lanes in ForEach card


### Refactor

- Use BTreeMaps for Modules for fixed ordering of keys

- Shorten unnamed CompositeCard name

## v0.1.75


### Features

- If/Else cards take another Card as parameter instead of lanes


### Refactor

- CompositeCard names are optional

## v0.1.74


### Break

- Only publish the web target to npm

## v0.1.70


### Refactor

- Implement Default for OwnedValue

- Do not take ownership of OwnedValue when inserting


### Break

- Use structs for the inner OwnedValue::Object representation

## v0.1.69


### Features

- Allow constructing FieldTables from iterators

- Add OwnedValues that allow users to save and load Values between VM instances


### Refactor

- Implement Clone for KeyMap and CompiledProgram

## v0.1.68


### Refactor

- Disallow null field when deserializing modules

## v0.1.67


### Bug Fixes

- Fix serialization of modules


### Refactor

- Honor the cargo target dir environment variable if present in the C API

## v0.1.66


### Bug Fixes

- Fix Module serialization

## v0.1.65


### Bug Fixes

- Fix potential lifetime issue in the Python wrapper


### Features

- Support `null` values in Module deserializing

## v0.1.64


### Bug Fixes

- Fix the python CompilationUnit parsing and storage

## v0.1.63


### Refactor

- Allow empty `lanes` when parsing `Modules`

- Use Cow in CaoProgram

## v0.1.62


### Bug Fixes

- Jumps now work within a namespace

- Fix clippy warnings


### Refactor

- CaoPrograms are now Modules

- Split public and internal Lane data

## v0.1.61


### Features

- Add `Noop` card that does nothing

- Introduce modules


### Refactor

- Borrow local variables in the compiler

- Add &str indexing to KeyMap

- Rename compiled program to CaoCompiledProgram

## v0.1.60


### Refactor

- Remove python 3.6 support

- Drop the lifetime requirement for `register_function`

## v0.1.57


### Features

- Add pop_n to ValueStack

- Add composite cards


### Refactor

- Do not inline get_desc

- Simplify SubProgram by using owned types instead of borrows

## v0.1.56


### Bug Fixes

- Undefined behaviour when decoding trivial structs

- Fix memory leak when using Tables


### Refactor

- Return error on invalid key in KeyMaps

## v0.1.55


### Bug Fixes

- Remove custom deserialization for Variable names


### Refactor

- Replace variable names HashMap with KeyMap

## v0.1.54


### Bug Fixes

- Use power of two capacity when deserializing

## v0.1.53


### Bug Fixes

- Yet more map serialization

## v0.1.52


### Bug Fixes

- Fix cbor serialization of KeyMap

## v0.1.51


### Bug Fixes

- Fix binary serialization of KeyMap


### Features

- Add traces to runtime errors

## v0.1.48


### Refactor

- Remove Instruction from the public interface

## v0.1.42


### Features

- Pretty print inner error in Subtask failures


### Refactor

- Use tilde deps

## v0.1.40


### Features

- Add basic program running to the C api

- Add string insertion to VM


### Refactor

- Derive Default for CaoIr, hide Local from the public api

## v0.1.39


### Bug Fixes

- Add missing card to the schema


### Features

- Pass in `i` to `Repeat`-ed Lanes


### Refactor

- Use different constant for 0 inputs to Handle

- Use ABI3 in the python interface

## v0.1.38


### Features

- Bad arity in for-each lane is an error


### Refactor

- Unsafe get_str/as_str methods for String values

- Put VarNode behind a pointer to reduce size of Card

- JumpErrors will return LaneNodes instead of strings

- Fix the ForEach node jumperror message wording


### Styling

- Use Titlecase for # Safety sections in docs


### Wip

- Add nested_foreach test + refactor

## v0.1.37


### Refactor

- The compiler no longer takes ownership of IR

## v0.1.35


### Bug Fixes

- Fix changelog format

- Add Len and ForEach cards to descriptions


### Refactor

- Impl From<CompileOptions> instead of Into

- Return values instead of references from FieldTable

## v0.1.33


### Features

- Len card

- ForEach card


### Refactor

- Hide Card::instruction from the public interface

- Properties will use cards as inputs

- SetProperty will take the value as the last parameter

- Repeat card refactor/optimization

## v0.1.32


### Bug Fixes

- CMakeLists builds cao-lang core using the correct configuration


### Refactor

- Move Lane into its own file

- Hide unneeded stuff from the interface

- Init cargo xtask

<!-- generated by git-cliff -->