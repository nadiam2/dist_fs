# dist_fs

This is a WIP project to implement a safe and resilient distributed filesystem as well as various applications on top of it. Before continuing, it is worth noting that this project is mostly a learning experience for me to advance my Rust skills, implement a modified form of the actor model, and build a robust distributed system. With that, it likely will never reach a state where it could be used in a production environment. Regardless, at the time of writing this file, practically all filesystem operations are complete, but very little beyond it exists.

## Repo Layout

There are two different places where code can be found: `scripts/` and `src/`. 

#### scripts/

`scripts/`, as its name would indicate, contain numerous helper files (which are not very generic at the moment) for replicating code across machines and viewing logs.

#### src/

`src/` contains all the Rust source code of the project. The filenames should indicate what the file does, and at the moment, the majority of the code is modular without much need for coupling.

## Design

Each module of the system is usually encompassed in an entire source code file (like `src/filesystem.rs` for example) and is implemented through a modified actor model. More specifically, each module introduces new messages that can be sent between members of the system. These messages contain a few common features. They must all

- implement the `OperationWriteExecute` trait found in `src/operation.rs` which allows the message to be sent across networks as well as defines what the recipient should do upon receiving the message. 

- be present inside the `try_parse_buf` function in the same file to allow reading the operation from a socket. 

With these satisfied, the message should be ready to bounce between the members of the system.

## TODOS

- Obfuscated filenames
- Optional file encryption
- Applications on top of the filesystem
- More robust scripts
- Improvement on the `try_parse_buf` function - can it be generic from an implemented trait/build.rs magic?

## Inspiration

During my time at UIUC, one of my favorite courses was CS425: Distributed Systems where we implemented something similar in the language of our choice. I chose Rust and had to learn it as I worked through the MPs, which led to some pretty shady design choices. This project started as a reimplementation (with improved design choices) in my free time but turned into a passion project I have continued to work and expand on.
