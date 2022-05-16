# ThesisOS #

# Table of Contents
1. [About](#About)
2. [Building](#Building)
4. [Testing](#Testing)

# About 
The linux operating system is one of the most popular operating system(OS) nowdays, however it has become too complex for teaching OS fundamentals. Furthermore there has been a push for using Rust instead of C for writting more robust OS kernels. 
The goal of this project is to investigate recent Educational Operating Systems with a specific focus in Rust.
# Building
This project runs only in Linux distributions
## Set Up Environment

Install rustup by following the instructions at https://rustup.rs.

* Rust nightly
  * You need a nightly compiler to run the code in this repository.
  * Set the nightly compiler for the current directory by running ``rustup update nightly --force to update`` to update to the latest nightly even if some components such as ``rustfmt`` are missing it.
  * You can check that you have a nightly version installed by running rustc --version: The version number should contain -nightly at the end.
  * This project has been tested runing on rustc 1.62.0-nightly (60e50fc1c 2022-04-04).
* QEMU >= 6.2.0
* For this project the bootimage tool is needed install it using:
  * ``cargo install bootimage``
* In order to run the bootloader you need to have ``llvm-tools-preview`` installed
  * you can do so by running ``rustup component add llvm-tools-preview``  


## How to run

```bash
$ git clone git@github.com:mario-ah-salamanca/ThesisOS.git
$ cd ThesisOS
$ cargo install bootimage
$ rustup component add llvm-tools-preview
$ cargo bootimage
$ cargo run
```

# Testing

A test framework is included with Thesis OS you can run all test using ``cargo test``
or by specifying the test name using the  ``--test`` flag


