# rusty-nussknacker

This project is simplified implementation of [Nussknacker](https://github.com/TouK/nussknacker) scenario runtime in Rust.

The idea is to take JSON representation of Nussknacker scenario (basically, a flow diagram that can describe event stream processing, 
or request-response decision tree) and be able to parse it and run on simplified JSON data. The scenario may look like this:
![Sample scenario](https://nussknacker.io/documentation/assets/images/nu_scenario-9438bb1c2a859a1475d09244c975e462.png)

In standard, JVM version of Nussknacker the expression language and custom components are pluggable via special API (and dynamic classloading).
We also provide runtimes which read data from Kafka or expose REST endpoint. This project provides only library which 
can serve as a base for such a service, and a simplistic console app, which reads JSON data from stdin.

At the moment, Javascript expressions and a simple for-each custom component is provided. I hope it will be possible to load other stuff
e.g. with dlopen.

Now, this is my first Rust project, so for sure there places where it smells Scala/JVM. Some of the things I'm sure can be improved:
- Errors using some crate that would reduce the boilerplate
- Constants with lazy_static
- Restrict pub usage in modules and fields
- Tests for more error paths

There are also more things I'd like to work on:
- Asynchronous invocations are synchronous
- Typing of variables
- Handling Javascript expressions is certainly not optimal and a bit hacky.
- Join nodes (e.g. unions), custom sinks and sources are not supported
