This project was created to ease other's journeys in creating
their own custom async executors. The codebase respectifully derives
from other developers in futures-rs (the license is in their names). 
The aim is to provide the most absolute minimal waker system and provide 
clarity into how one of the most challenging parts of how async-await works.

To see a complete async-await and executor check out the [howdy example](https://github.com/richardanaya/woke/blob/master/examples/howdy/src/main.rs)

## How does Async-Await work?

This section is meant for people who are complete beginners.  

A **future** is an object that usually starts some long term activity (like a timer!).  It can be asked about its current state (polling), returning either "Ready!" or "I'm still working, don't bother me!".  Futures can depend on other futures to complete, creating a chain of polling.

An **executor** is an object that contains a list of futures that need to be polled. It goes through them one by one and asks "hey, are you done yet?" (polling). Each time a future is polled, the executor gives it a **waker** so that the executor can check back in on it later when the future says its ready! 

A **task** is simply a top most level future. Exectors will poll on a list of task futures that will poll their child executors. Don't worry though  `async { }` auto generates most of  that heirarchal future dependency magic!

A **waker** is an object that can later call a function on an object that will tell the executor "hey, put me back on your list of things to do!". It uses a lot of pointer magic and is argueably the most mind-bendy aspects of async-await (and the reason **woke** was written)!


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
woke = "0.0.1"
```
The current `async-await` feature requires Rust 1.39 or later.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in woke by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
