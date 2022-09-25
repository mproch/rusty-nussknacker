# rusty-nussknacker

This project is simplified implementation of [Nussknacker](https://github.com/TouK/nussknacker) runtime in Rust.

The idea is to take JSON representation of Nussknacker scenario (basically, a flow diagram that can describe event stream processing, 
or request-response decision tree) and be able to run the scenario on simple JSON input.

In standard, JVM version of Nussknacker the expression language and custom components are pluggable via special API (and dynamic classloading).
Here, we only provide simplified API, implementations would need separate Rust application, having rusty-nussknacker as dependency.

At the moment, Javascript expressions and a simple for-each custom component is provided. 

Some of the things that are ommitted (due to lack of time etc.):
- Detailed validation
- Typing of variables
- Join nodes (e.g. unions), custom sinks and sources are not supported
- Invocations are synchronous
- Handling Javascript expressions is not optimal and a bit hacky.
