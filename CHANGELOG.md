# Changelog
## v0.1.105


### Bug Fixes

- Traces now include their namespace ([3bc0f20](3bc0f2023cea9a9396fa70657ae6a4b5ce803fa2))

### Features

- Init the standard library ([29ee7c8](29ee7c89e1cba98e5a5d92afd97f0d7109a92991))
- Read property shorthands ([1d89e0c](1d89e0c7270537b8d89269649af7ad279101766d))
- Set property shorthands ([e347ad5](e347ad5c860e1d104c4d3392053536c398872904))

### Refactor

- Rename ForEach variable ([b5e3125](b5e31252cd84148e2234f69c0e7d4a16d52144ed))
- [**breaking**] Move the `value` parameter on `SetProperty` to the first place ([a61a936](a61a9362689099c33d551e2653834a84d3bc6f66))

### Break

- Remove subprogram desc ([f641a01](f641a017ed75dc938a6ada81737095875502363c))
## v0.1.104


### Features

- Add Function values, redefine Jump card in terms of Function value ([7420433](7420433966fc66f8a75dd0d73b8b856d54d6e6b3))
- DynamicJump card ([b509962](b509962912edbee27874cb87f6656d388a16c700))

### Refactor

- Manual PartialOrd impl for Value ([33fe786](33fe786c8b56376cb5040502ed3a1211d270986b))
## v0.1.103


### Features

- Add _mut lookup methods ([4b1e06e](4b1e06e64c82249b82ada74d0369e8aa966c2aa6))

### Refactor

- Factor out ForEach card body ([36f48ea](36f48ead563bf0edebbc828eb7dcc597ab844d85))
- Use Strings in cards to simplify the interface ([5088e16](5088e16c327d85d1b90858096610452b937ac51c))

### Break

- Swap function argument popping, first argument is popped last. This is consistent with native function calls. ([b3f4944](b3f49448c17ff72b6bbcf8a2c498cd77d3f65dad))
## v0.1.102


### Features

- Add submodule and lane lookups ([6a03afc](6a03afcc9420b7bcaa983047f584fb904b74e241))

### Refactor

- Remove StringNode ([3c018df](3c018df17be1065a88adc7bc18cf20a1a4bcdb38))
- Remove LaneNode ([d52dada](d52dada8f62baa58e13175c6d3ddb7e576eae83e))
## v0.1.101


### Refactor

- Simplify Float scalars ([4887549](4887549a9a8bc016f79d4280d789b4cd64491488))
- Use vectors for submodules ([972328c](972328c300a427da93a24c700020f192aefbf657))
- Use vectors for lanes ([df60208](df6020872b179e8e9f7b516b7bb2821e44a942f8))
## v0.1.100


### Bug Fixes

- Fix child indexing of loop cards ([b1aa8ce](b1aa8ce7e4dcda58758d16d74ee6fa34d6817fcd))

### Refactor

- Remove as_card_list functions ([e47c6d8](e47c6d878f1bbb5884f4a04c90f9393532cd9dcc))
- Use child array for IfElse ([11b64a2](11b64a22443438668a49fbd5e3e25f0e0a8f906e))
- Use child array for While cards ([caf18ae](caf18aee5e86184657092046b943dd3b240930df))
- Remove the IntegerNode ([df6f5de](df6f5deae60f13e1d376dca83ea280e98de742f2))
## v0.1.99


### Refactor

- Drop Object var and add the Value var for ForEach ([d3a8fbc](d3a8fbc64c8dbb0eff06d747a2595148c0855242))
## v0.1.98


### Refactor

- Remove the `name` property of `CompositeCard`s ([9e9c782](9e9c78218db2ac2f05741b82750a4240e1b256dc))
## v0.1.97


### Bug Fixes

- Raise error on ambigous error ([b6270fe](b6270fed2a1e6cda6e9eaa24440ca5ceb7454269))

### Features

- Repeat cards take a sub-card as input ([133da41](133da41ece0c0fb94df6c2ce630d5343d8aa7895))
- Take Card as ForEach body ([21ae812](21ae8129c38245610f7c0f7f9600e9ea765fba8c))
- Allow specifying the `i` `k` and `o` parameters of ForEach ([9308486](93084866307cadee201a2728ca5bbb6e6a7b40d8))
- Take a sub-card as input for ForEach ([8ac2d2d](8ac2d2d710d455ea4ccd3d3db9908b722a06f4a1))

### Refactor

- Use vector for imports collection ([21a3933](21a3933bb1f75fc81ec741221ea2a25e11fb4cdc))
- Use vector for imports collection ([b4587fa](b4587fa53a5f372b6715f52d9a6780ca003ae63f))
## v0.1.96


### Bug Fixes

- Fix assigning to the same local variable multiple times ([f960ff2](f960ff2e29ea477d0e236390527e12514667f9ed))

### Features

- Add while card ([9834064](98340640578fd2a68ab34481ec2d404c686eb3fb))
## v0.1.95


### Features

- Add stack trace to runtime errors ([8982282](8982282c7aa9d8ec3eae84676622437f916937ae))
- Allow field_tables to be queried by str and int ([acb8373](acb8373eec4b3868b7f9b89cd23905a080207dbd))
## v0.1.93


### Features

- Use the module level CardIndex for compile/runtime error tracing ([5eb28b3](5eb28b372f6543a11f624e2b82b37297d5b751a0))

### Refactor

- Remove the `noop` card. We already had `pass`... ([d4aa001](d4aa001b893f7bf76ef52d63251a4c86ca0b2ed3))
- Remove the `pass` instruction ([e384915](e3849153c4ef2e860c46cd5f79d2816623fc791d))
## v0.1.92


### Features

- Remove cards by index ([bbdf6a0](bbdf6a07b3211f90669ccac688f1545d2d42f771))
- Insert card at index ([b59de0f](b59de0f353de648e7a563ec6e1696c4d4cc241c5))
- Add set_card API ([d414268](d414268a0827c17557516c3841c5e6dc96543cf5))

### Refactor

- Rename `replace_card` and return the old `Card` on success ([7fc4f22](7fc4f22ccce6849146f32cfd27d6be041061fe92))
## v0.1.91


### Features

- Add `ty` to `CompositeCard` metadata ([9e5ce5f](9e5ce5fe29e9fe392218bb3705901d6a9c873f9f))
## v0.1.90


### Bug Fixes

- Fix with_sub_index on LaneCardIndex ([c96aad0](c96aad0b15a66d1e30aa8ebeb5c8bc09984d41ae))

### Features

- Current index api for LaneCardIndex ([a8fddf0](a8fddf0c4186fdf77d14c4ee99de3db885f388b7))
- All sub-cards are now indexable, not just CompositeCards ([be627cd](be627cd3c852028eebe59f121011ceb47c87a919))

### Refactor

- Use owned string in CardIndex to simplify lifetimes ([db31937](db31937af208070a167e528da3b2b92912e5187c))
- Simplify the card index data model ([22ea13f](22ea13fdcb0571701d6bd621ffa8429f852293eb))

### Break

- Remove LaneCardIndex::default ([5894641](58946412d8ce084d32f49a82b1663368095df544))
## v0.1.89


### Features

- Current index api ([20e2a62](20e2a629026f584cfaab8d485be39608e4ac22e8))
## v0.1.88


### Features

- Add composite_card ctor ([e3b68ee](e3b68eecf4a7d535a24f91eac8b70ed005ed0d61))

### Refactor

- Nicer with_sub_index api ([f671039](f6710397962e8697c3b2d38b073ce0fb517e6a13))
## v0.1.87


### Features

- Expand Card Index API ([60412d4](60412d4a3da9c51f2fc63438d73897f64a60835a))

### Refactor

- Use a boxed struct for nested cards to reduce Card size ([90f1525](90f1525307ced7e768a8d16166252a79d0009e34))
## v0.1.86


### Features

- Add Module query API to fetch concrete cards ([d8e7794](d8e779450b5506c1c99355aba1d514538c842843))
## v0.1.85


### Break

- Revert card splitting, just inline cards... ([dd32173](dd321731be2a324b65f090b736aa13feb0185806))
## v0.1.84


### Refactor

- Remove lifetimes from the public Module ([b10c9dc](b10c9dcb720c9b09cad23b1ee38eeae7fb54c423))
## v0.1.83


### Features

- Decouple cards and lanes, cards are no longer inlined in lanes ([9258ce9](9258ce999b42bf30ba906121c4804dd33e731335))
## v0.1.82


### Break

- Simplify Module serialization format ([068ae75](068ae75da60124fe5dce158445c1859bd97617f5))
## v0.1.81


### Features

- Add submodule imports ([8cae2af](8cae2afa9740d1dfeebefd5ff81791af4de069f2))
- Add super function imports ([2ca8abc](2ca8abc8ee412fe201553cd7fc5cd01fb934c60e))
- Add super module import ([d82c4ae](d82c4aea273615e1e572a03b51a4a226fce5e137))

### Breaking

- Reserve the `super` keyword ([44c72dd](44c72dd97496b6f7b6021aa676e306715c2091e1))
## v0.1.80


### Features

- Add imports ([47f5697](47f5697f7665a241155b02722ac8ee322470c067))

### Break

- Remove the `compute_stack_at_card` function for now ([84f0ce8](84f0ce8f5ae1acecef634abe4c3352fad5cc878b))
- Remove Lane lookup in parent scopes ([9fc7fd5](9fc7fd5c3daa8544e7012ae778e092b292172ea0))
## v0.1.79


### Bug Fixes

- Fix undefined behaviour in the Compiler ([7423c9e](7423c9e0a2713bc8c940eb1b77053be406cec7b8))
- [**breaking**] Fix leaky objects in RuntimeData ([6805ba6](6805ba64a246bb3a6e4717ebd2ffce5a13913fe0))
## v0.1.78


### Features

- Add `compute_stack_at_card` to Lanes ([df2bba6](df2bba6529ccac00f8b839c5a0a80c82b0aa0889))
- Add recursion limit to compile options ([2dd0c28](2dd0c280d1cfd6b1e4386b141f3dd6e07cb3bb7e))

### Refactor

- [**breaking**] Rename hashing function ([cbea3d2](cbea3d2b48599acdca7a3c2c34fd35bb7e2221c5))
## v0.1.77


### Features

- Add a hashing function to Programs ([76e5170](76e5170807664c9afb4ae3237415937e7e45e647))
## v0.1.76


### Bug Fixes

- Fix panic on empty target lanes in ForEach card ([0cce9fd](0cce9fd4f32b2a1fc3f19a216db6cd52d8dd2be2))

### Refactor

- [**breaking**] Use BTreeMaps for Modules for fixed ordering of keys ([a4a89a4](a4a89a49620b35fe18856e6b033f2e798f5a5c94))
- Shorten unnamed CompositeCard name ([ce6f784](ce6f7845d2b4b13745e9a49b465aac854fc5726d))
## v0.1.75


### Features

- [**breaking**] If/Else cards take another Card as parameter instead of lanes ([26c5e11](26c5e117e836837d72fb0dbfafb482cbe77c16a3))

### Refactor

- [**breaking**] CompositeCard names are optional ([7065675](70656754e663d82bdbbeb29ce677c7fd77676829))
## v0.1.74


### Break

- [**breaking**] Only publish the web target to npm ([54c39fb](54c39fb945e8b230d0ecfb7d6591da5a2f91d225))
## v0.1.70


### Refactor

- Implement Default for OwnedValue ([3fcdb00](3fcdb0021d581acfb45705ef9e58ba046f3f0b04))
- Do not take ownership of OwnedValue when inserting ([db142a7](db142a71ec3fc3bb5d1e6ef327b436a05aa18c83))

### Break

- Use structs for the inner OwnedValue::Object representation ([25953e7](25953e73e08f0566c014b0d53ac7bb2912715104))
## v0.1.69


### Features

- Allow constructing FieldTables from iterators ([e8cf6e1](e8cf6e1044758adfa0c6767a3d74bdcfbffd456e))
- Add OwnedValues that allow users to save and load Values between VM instances ([a33ba9d](a33ba9df4e1f5b855efe2d123e5fc1e92774ff9d))

### Refactor

- Implement Clone for KeyMap and CompiledProgram ([8809f99](8809f9965995baf4269aa0c0243f69732daa5b39))
## v0.1.68


### Refactor

- Disallow null field when deserializing modules ([f66de49](f66de49b845cd4f5cf160b887f4aa11b6e907541))
## v0.1.67


### Bug Fixes

- Fix serialization of modules ([6c78661](6c78661d5284e71c255e635a5b51b877fd27a6a6))

### Refactor

- Honor the cargo target dir environment variable if present in the C API ([9021739](90217397533303e0d3dbe4de467a1ee5b6ee27d9))
## v0.1.66


### Bug Fixes

- Fix Module serialization ([dba1885](dba18852aa6de638d2f9277cad746dbec798b007))
## v0.1.65


### Bug Fixes

- Fix potential lifetime issue in the Python wrapper ([046f98d](046f98d61325034bfd5b442e1bfb5d4ea50855de))

### Features

- Support `null` values in Module deserializing ([09c6d89](09c6d89df2ddb7cf4f66d6df67c843abb45768a7))
## v0.1.64


### Bug Fixes

- Fix the python CompilationUnit parsing and storage ([bafd979](bafd9794e11b908ca8f83bd43cdf2f6b44194260))
## v0.1.63


### Refactor

- Allow empty `lanes` when parsing `Modules` ([dce15f7](dce15f7a483ae46c93a69320c390aa882045bf50))
- Use Cow in CaoProgram ([4fc6ac9](4fc6ac9044c699473c6f28b71f938e5b6eb0e5d1))
## v0.1.62


### Bug Fixes

- Jumps now work within a namespace ([4abbdc1](4abbdc1c56d7610162038becb0f9c7a1683a63b7))
- Fix clippy warnings ([760c8e5](760c8e580bdbbb08c17a4ce59ea72635a730022c))

### Refactor

- CaoPrograms are now Modules ([4baf3d2](4baf3d2323cbeb80efd2543a92aa0ef32e46f149))
- Split public and internal Lane data ([f5c2d23](f5c2d235c0ba0fb83e90c191657746a039fd5991))
## v0.1.61


### Features

- Add `Noop` card that does nothing ([2187479](2187479203a15171cf51357cc64f4bb8e13425a7))
- Introduce modules ([424b9e4](424b9e47cccbc97cd35f5dc390683b017d17b0ba))

### Refactor

- Borrow local variables in the compiler ([68936e0](68936e06e8fa49d40bb5cffe3fcb91745f0678bf))
- Add &str indexing to KeyMap ([b2d1e8b](b2d1e8bb3622aae8ee55dcf25a78d746ace03ad6))
- Rename compiled program to CaoCompiledProgram ([04cf281](04cf28130a3ba5d35a0b9be4a59a1dfa9f7461ce))
## v0.1.60


### Refactor

- Remove python 3.6 support ([46e4017](46e40171a838099c0cbe29c7802907ba50d61072))
- Drop the lifetime requirement for `register_function` ([cb12822](cb12822c52f7909c216380df18b490d415947414))
## v0.1.57


### Features

- Add pop_n to ValueStack ([979b121](979b121f313f1bcd148478a30eba07b1d65ac1e7))
- Add composite cards ([7a59afc](7a59afc1392ac415fdbe8924937b646a4afadcd0))

### Refactor

- Do not inline get_desc ([29321a5](29321a59e785d83c1ad1c9e99e32f8af46e6d7b0))
- Simplify SubProgram by using owned types instead of borrows ([a49e154](a49e154f7c12ff5f89d097624e01cd3a83df2961))
## v0.1.56


### Bug Fixes

- Undefined behaviour when decoding trivial structs ([c53ff32](c53ff32deff25802f83297126e022bcdef6c4af9))
- Fix memory leak when using Tables ([a4ab5ba](a4ab5ba7962ba077a0504a18285a4bd1edf52737))

### Refactor

- Return error on invalid key in KeyMaps ([f683f12](f683f124deb7d0e3f971c80aa906186a5962dc6b))
## v0.1.55


### Bug Fixes

- Remove custom deserialization for Variable names ([309eccd](309eccd498d7f3ca64c0e2b6ff48444a6353a483))

### Refactor

- Replace variable names HashMap with KeyMap ([064c9c9](064c9c98d2969794d49943ec1c3210beb523da22))
## v0.1.54


### Bug Fixes

- Use power of two capacity when deserializing ([2ff15ad](2ff15adf53623b851f4e002a803557d3026b2c7d))
## v0.1.53


### Bug Fixes

- Yet more map serialization ([448b83a](448b83ad08f20096dd658cc3ff7719ea97e6bd40))
## v0.1.52


### Bug Fixes

- Fix cbor serialization of KeyMap ([9d17735](9d177351f74be3ac23d7e3078badce4c1610c787))
## v0.1.51


### Bug Fixes

- Fix binary serialization of KeyMap ([de02dfb](de02dfb087c56433203932c1450d832d25a8563c))

### Features

- Add traces to runtime errors ([3d763ff](3d763ff1cc436b866785ebd60ea88b289d7e0b07))
## v0.1.48


### Refactor

- Remove Instruction from the public interface ([3f15d4c](3f15d4ccd34a03fa249efd7b7fa12704622fd05c))
## v0.1.42


### Features

- Pretty print inner error in Subtask failures ([5bab37f](5bab37fa00471903c6e47c72ed9cfa2a6a47a790))

### Refactor

- Use tilde deps ([d2a2a3c](d2a2a3c893417fa7e41e9914e2984d19abbcde15))
## v0.1.40


### Features

- Add basic program running to the C api ([70670e3](70670e39b7bed872e239d67bc4a15bc045d3cf79))
- Add string insertion to VM ([00ac3fc](00ac3fc73833d1848805fd28ddd674f744888d06))

### Refactor

- Derive Default for CaoIr, hide Local from the public api ([6db1547](6db1547d5c81d1d3fbc0b281596c91442f2e4470))
## v0.1.39


### Bug Fixes

- Add missing card to the schema ([09fb0bf](09fb0bf4106a42aa13f6aa58673ca4447130bcde))

### Features

- Pass in `i` to `Repeat`-ed Lanes ([d486d20](d486d20cbab1633ad74a375c38ce2fc0670add9f))

### Refactor

- Use different constant for 0 inputs to Handle ([9cf93d3](9cf93d37c809d17a6c07aaf024ffe146e8f01f75))
- Use ABI3 in the python interface ([018e105](018e105b6e30e689db57f9af00028d7a2c950598))
## v0.1.38


### Features

- Bad arity in for-each lane is an error ([6d2dd42](6d2dd42a850c0dc388511b73d36a14d5450cbd0e))

### Refactor

- Unsafe get_str/as_str methods for String values ([bba6398](bba63986cd14f7f34824982405b8553c53b252fc))
- Put VarNode behind a pointer to reduce size of Card ([61b5f34](61b5f348fd5b8c177f9ad7bfcadc26147b4a728e))
- [**breaking**] JumpErrors will return LaneNodes instead of strings ([b55a13d](b55a13dc02a04c0c7f591a31a9f3ffbe0ad42694))
- Fix the ForEach node jumperror message wording ([e015430](e0154308360ca8a00a4906b6b5c8e9cc2737848c))

### Styling

- Use Titlecase for # Safety sections in docs ([58bc5cf](58bc5cfe14adc4ca927bc0476daffefa7a012ebe))

### Wip

- Add nested_foreach test + refactor ([9975375](9975375881ea87e04ac825bb57ab0d96a61af1ec))
## v0.1.37


### Refactor

- [**breaking**] The compiler no longer takes ownership of IR ([2b81a4e](2b81a4ecfe43e2987b131dcaac6570e4399756cb))
## v0.1.35


### Bug Fixes

- Fix changelog format ([4489e30](4489e303f318e78e9646334028bb89ab43f8c6d4))
- Add Len and ForEach cards to descriptions ([274cd6e](274cd6e3a1028b401ac7b0d0beb363fb8e00b9a0))

### Refactor

- Impl From<CompileOptions> instead of Into ([e74290b](e74290b54735f44f386fdd134fce3b586ff8f610))
- Return values instead of references from FieldTable ([6e8e6fc](6e8e6fc1c7264967d3dc3c65041c571100ad5c79))
## v0.1.33


### Features

- Len card ([2f84785](2f8478501bc2228a4484c3791e3ac061490acbd1))
- ForEach card ([be73981](be73981a629ab0e74c1bf834563bfe3e6000ead7))

### Refactor

- [**breaking**] Hide Card::instruction from the public interface ([4fd885d](4fd885dd72b6ae8a4248b10cf299eefba58bfec1))
- [**breaking**] Properties will use cards as inputs ([63f57bb](63f57bb38358decea8064e9f30a367d14b1199e2))
- [**breaking**] SetProperty will take the value as the last parameter ([c53d1a7](c53d1a7967a2dee64618a31572a8958f83646e71))
- Repeat card refactor/optimization ([b47f9bc](b47f9bc5f310e4cd83823c5e77e3560481e81053))
## v0.1.32


### Bug Fixes

- CMakeLists builds cao-lang core using the correct configuration ([bb2fd18](bb2fd18dd9258404a9db371bd360b75af6e6570c))

### Refactor

- Move Lane into its own file ([612f0a7](612f0a722ec7627ef684dab64c95dbaa67b4953c))
- [**breaking**] Hide unneeded stuff from the interface ([d9903fe](d9903fea945c8dc0b3a73b971d48d00b3f9dd4d6))
- Init cargo xtask ([098203d](098203d3036856577cca4b7fa6b9a84f6a5c2431))
<!-- generated by git-cliff -->