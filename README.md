# Akio

Akio takes heavy inspiration from the Erlang VM and other actor frameworks
such as Akka and Orleans. The goal of this project is to take the ideals
of these actor frameworks and apply them to rust, where we can have
extremely high performance while providing an friendly, type safe API
without the overhead we see in other languages. Akio will be a platform 
for building highly concurrent, resilient, and scalable distributed systems 
in rust.

## Features

* [x] Local Actors
* [x] Type-safe API
* [ ] Networked Actors
* [ ] Clustering System
* [ ] Persistence

## Requirements

Akio currently requires nightly rust. This project is waiting on the following 
language features to be stabilized:

* `proc_macro`
* `conservative_impl_trait`
* `fnbox`

