## What is Miri?
[Miri](https://github.com/rust-lang/miri) is a tool for detecting undefined behavior in Rust programs, capable of identifying unsafe code practices such as out-of-bounds memory accesses, use-after-free errors, and invalid use of uninitialized data. It also checks for violations of intrinsic preconditions, type invariants, and memory alignment issues, while detecting data races and experimental aliasing rule violations. Additionally, Miri highlights memory leaks by flagging unreferenced allocations at the end of program execution.

## Miri's Role on Rust Safety Analysis
Miri is essential when there are unsafe code dependencies throughout the entire program, including external libraries.
It ensures that these dependencies do not introduce memory safety issues.

## How to run Miri for Islet RMM

To analyze Islet RMM, run:
```bash
./scripts/tests/miri.sh
```

## Employing Miri for Islet RMM Safety Checks
Miri operates in an _Interrupt-on-Violation_ manner, immediately halting execution when a memory safety violation is detected. This non-continuable approach necessitates resolving each issue individually before proceeding with further analysis, which can make comprehensive testing time-consuming and complex. Furthermore, since Miri does not simulate assembly code, its analysis scope is limited, potentially overlooking issues in mixed-codebases that heavily rely on FFI or inline assembly. These factors collectively increase the engineering burden when integrating Miri extensively into a project.  
Considering these limitations, the following implementation guidelines are proposed:

### Inline Assembly Handling Strategy
Using conditional compilation enables Miri to bypass unsupported assembly instructions, allowing the desired checks to proceed.
However, our project involves approximately 70 system registers, along with numerous local registers and caching mechanisms that rely heavily on assembly.  
To address the challenges of handling inline assembly with Miri, we can consider two approaches: 1) Conditional Compilation for Inline Assembly and 2) Implementing a Stubbing Feature for Miri. However, both methods can reduce code readability. Therefore, we opt for 3) Adding an Assembly Crate Layer as a more maintainable solution.

Applying conditional compilation throughout would make the main codebase less readable:  

1. Conditional Compilation for Inline Assembly  
   ```rust
   fn main() {
       #[cfg(not(miri))]
       call_asm();
       dangling_pointer();
   }
   ```

2. Implementing a Stubbing Feature for Miri  
Related Issue: https://github.com/rust-lang/miri/issues/3729  
ref) https://model-checking.github.io/kani-verifier-blog/2023/02/28/kani-internship-projects-2022-stubbing.html  
   ```rust
   #[cfg(kani)]
   fn mock_asm<T: kani::Arbitrary>() -> T {
       kani::any()
   }

   #[cfg(kani)]
   #[kani::proof]
   #[kani::stub(std::arch::asm, mock_asm)]
   fn my_function_with_asm() {
       unsafe {
           std::arch::asm!("mov {0:r}, 5", out(reg) x);
       }
   }
   ```

3. **Creating an Assembly Crate Layer**  
All assembly code manipulating system registers is encapsulated within the `armv9a` library. By providing a dummy feature, we can maintain a consistent codebase for RMM while excluding inline assembly during Miri analysis. This approach enables the application of Miri without significantly modifying the main codebase.

### Writing Additional Test Code for Miri
The current test codes are implemented in Rust by rewriting the C-based ACS unit test codes as shown below.  
Test codes are implemented at the end of the source code files where the functionality is implemented, whenever possible.
```rust
1 #[cfg(test)]
2 mod test {
3     use crate::granule::GRANULE_SIZE;
4     use crate::realm::rd::{Rd, State};
5     use crate::rmi::*;
6     use crate::test_utils::{mock, *};
7
8     use alloc::vec;
9
   ```
- As shown in lines 1 and 2, the configuration is set to build only for testing purposes, and the test module is defined.
- As shown in line 6, mockup modules required for testing are included from the `test_utils` module.

```rust
 10     // Source: https://github.com/ARM-software/cca-rmm-acs
 11     // Test Case: cmd_rtt_create
 12     // Covered RMIs: RTT_CREATE, RTT_DESTROY, RTT_READ_ENTRY
 13     #[test]
 14     fn rmi_rtt_create_positive() {
 15         let rd = realm_create();
```
- As shown in line 13, the `#[test]` attribute is declared to generate each test case as a test binary name.
- As shown in line 15, if a realm needs to be created, the `realm_create()` function from the mock module is utilized.

```rust
 16
 17         let (rtt1, rtt2, rtt3, rtt4) = (
 18             mock::host::alloc_granule(IDX_RTT_LEVEL1),
 19             mock::host::alloc_granule(IDX_RTT_LEVEL2),
 20             mock::host::alloc_granule(IDX_RTT_LEVEL3),
 21             mock::host::alloc_granule(IDX_RTT_OTHER),
 22         );
```
- As shown in lines 18–21, for memory that needs to be shared from the Host, use `mock::host::alloc_granule()` to allocate memory.
```rust
 23
 24         for rtt in &[rtt1, rtt2, rtt3, rtt4] {
 25             let ret = rmi::<GRANULE_DELEGATE>(&[*rtt]);
 26             assert_eq!(ret[0], SUCCESS);
 27         }
```
- As shown in line 25, RMI or RSI command calls are made using `rmi::<{RMI-COMMAND-NAME}>({arguments})`.
- As shown in lines 26, 28, and 52-53, after making an RMI or RSI call, use assertions to verify the results.
```rust
 28
 29         let test_data = vec![
 30             (rtt1, 0x0, 0x1),
 31             (rtt2, 0x0, 0x2),
 32             (rtt3, 0x0, 0x3),
 33             (rtt4, 0x40000000, 0x2),
 34         ];
 35
 36         unsafe {
 37             let rd_obj = &*(rd as *const Rd);
 38             assert!(rd_obj.at_state(State::New));
 39         };
 40
 41         for (rtt, ipa, level) in &test_data {
 42             let ret = rmi::<RTT_CREATE>(&[rd, *rtt, *ipa, *level]);
 43             assert_eq!(ret[0], SUCCESS);
 44         }
 45
 46         let (rtt4_ipa, rtt4_level) = (test_data[3].1, test_data[3].2);
 47         let ret = rmi::<RTT_READ_ENTRY>(&[rd, rtt4_ipa, rtt4_level - 1]);
 48         assert_eq!(ret[0], SUCCESS);
 49
 50         let (state, desc) = (ret[2], ret[3]);
 51         const RMI_TABLE: usize = 2;
 52         assert_eq!(state, RMI_TABLE);
 53         assert_eq!(desc, rtt4);
 54
 55         for (_, ipa, level) in test_data.iter().rev() {
 56             let ret = rmi::<RTT_DESTROY>(&[rd, *ipa, *level]);
 57             assert_eq!(ret[0], SUCCESS);
 58         }
 59
 60         for rtt in &[rtt1, rtt2, rtt3, rtt4] {
 61             let ret = rmi::<GRANULE_UNDELEGATE>(&[*rtt]);
 62             assert_eq!(ret[0], SUCCESS);
 63         }
 64
 65         realm_destroy(rd);
 66
 67         miri_teardown();
 68     }
 69
```
- As shown in line 29, input values for testing can be defined.
- As shown in line 65, after creating a realm, the `realm_destroy()` mock function must be called.
- If test code involves mapping for RMM page tables or Realm stage 2 page tables, as shown in line 67, call the `miri_teardown()` function.
```rust
 70     // Source: https://github.com/ARM-software/cca-rmm-acs
 71     // Test Case: cmd_rtt_init_ripas
 72     // Covered RMIs: RTT_INIT_RIPAS, RTT_READ_ENTRY
 73     #[test]
 74     fn rmi_rtt_init_ripas_positive() {
 75         let rd = realm_create();
 76         let ipa = 0;
 77         mock::host::map(rd, ipa);
 78
 79         let base = (ipa / L3_SIZE) * L3_SIZE;
 80         let top = base + L3_SIZE;
 81         let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
 82         assert_eq!(ret[0], SUCCESS);
 83         assert_eq!(ret[1], top);
 84
 85         let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
 86         assert_eq!(ret[0], SUCCESS);
 87
 88         let (level, ripas) = (ret[1], ret[4]);
 89         const RMI_RAM: usize = 1;
 90         assert_eq!(level, MAP_LEVEL);
 91         assert_eq!(ripas, RMI_RAM);
 92
 93         mock::host::unmap(rd, ipa, false);
 94
 95         realm_destroy(rd);
 96
 97         miri_teardown();
 98     }
```
- As shown in lines 74-98, another test function can be defined.

