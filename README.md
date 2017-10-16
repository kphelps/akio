# Akio

[![Build Status](https://travis-ci.org/kphelps/akio.svg?branch=master)](https://travis-ci.org/kphelps/akio)

Akio is a fast, highly concurrent, and easy to use framework for building
distributed systems using the Actor Model.

**[API Documentation](https://kphelps.github.io/akio/akio/index.html)**

## Overview

Akio takes heavy inspiration from the Erlang VM and other actor frameworks
such as Akka and Orleans. The goal of this project is to take the ideals
of these actor frameworks and apply them to rust, where we can have
extremely high performance while providing an friendly, type safe API
without the overhead we see in other languages. Akio will be a platform 
for building highly concurrent, resilient, and scalable distributed systems 
in rust.

## Features

Akio is an early stage project under active development.

* [x] Local Actors
* [x] Type-safe API
* [ ] Supervision
* [ ] Networked Actors
* [ ] Clustering System
* [ ] Persistence

## Requirements

Akio currently requires nightly rust. This project is waiting on the following 
language features to be stabilized:

* `proc_macro`
* `conservative_impl_trait`
* `fnbox`

## Getting Started

To start using `akio`, add it to your `Cargo.toml`:

```
[dependencies]
akio = "0.1"
akio-syntax = "0.1"
```

### Hello World

Akio provides a few annotation-based macros to make working with actors very
simple. We'll walk through an actor-based "Hello World" to explain the basics
of Akio.

```rust
// akio depends on some nightly-only features in its code generation.
#![feature(conservative_impl_trait, proc_macro)]

extern crate akio;

// Akio provides a `prelude` module that will import everything you need.
use akio::prelude::*;

// First we define a struct that will hold the actor's state
struct ExampleActor {
    message: String,
}

// To implement an actor, we create a normal `impl` block, but annotate it
// with #[actor_impl]. This will generate some code to simplify message
// handler implementation and helpful methods on ActorRefs.
#[actor_impl]
impl ExampleActor {

    // We create a `new` constructor like any other struct.
    pub fn new() -> Self {
        Self {
            message: "Hello World".to_string(),
        }
    }

    // To define a message handler, we annotate a method with #[actor_api].
    // Under the hood, this generates code to make sending and receiving
    // messages look just like normal method calls - even over the network.
    #[actor_api]
    pub fn greet(&mut self) -> String {
        self.respond(self.message.clone())
    }

    #[actor_api]
    pub fn set_greeting(&mut self, new_message: String) {
        self.message = new_message;
        self.done()
    }
}

pub fn main() {
    // The core of akio is the actor system. We must create one to give our
    // actors a place to execute.
    let mut system = ActorSystem::new();

    // on_startup will run a function once the actor system is fully initialized.
    system.on_startup(|| {
        // We'll create an actor ref and tell it to start executing.
        let actor_ref = ExampleActor::new().start();

        // We can call our #[actor_api] methods directly on the ActorRef.
        // The generated code returns a Future so we do not have to block
        // while we wait for a response.
        let f = actor_ref.greet().and_then(move |message| {
            println!("{}", message);
            // We can also send a message without waiting for its response.
            // We generate methods prefixed with `send_` for this case.
            actor_ref.send_set_greeting("Good bye!".to_string());
            actor_ref.greet()
        }).map(|message| println!("{}", message));

        // There is also an actor-local context that allows us to execute
        // futures on the thread pool and access some context-specific data
        // like the current ActorSystem.
        context::execute(f);
        context::system().stop();
    });

    system.start();
}
```

## Contributing

All contributions are welcome. Please open an issue or pull request to get started!
