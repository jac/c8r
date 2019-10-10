# C8R
 C8R is a **C**hip**8** emulator made in **R**ust

 ![](space_invaders.gif)

 C8R runs in the terminal and implements all Chip8 features except for sound output.

 ### Building C8R
 The simplest way to build C8R is to use the Rust build tool Cargo.
 Clone the repository and run the following command in the project directory
 ```
 cargo build --release
 ```
 This should generate an executable **[project-root]/target/release/c8r**

 ### Running Chip8 Games
 To run games using C8R pass the path to a chip8 rom file

 Using Cargo:
 ```
 cargo run --release <path-to-rom> [clock-rate]
 ```
 Using executable:
 ```
 ./c8r <path-to-rom> [clock-rate]
 ```
Clock rate is an optional value which defaults to 500. It controls how many instructions are executed per second. Some games play better with a faster or slower clock rate
 ### Notes
 There does not exist a single chip8 standard. Some opcodes behave slightly differently between emulators due to conflicting standards.
 

The following instructions I implemented differently than the documentation I was following but in accordance with the test rom I was using
 * **8XY6** Shifts VX
 * **8XYE** Shifts VX
 * **FX55** Does not update register I
 * **FX65** Does not update register I