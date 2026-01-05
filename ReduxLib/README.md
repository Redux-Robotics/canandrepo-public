# ReduxLib

This is the repository for the official Redux Java/C++ vendordep.

## Installation
See https://docs.reduxrobotics.com/reduxlib.html for information on how to install and setup ReduxLib.

## Bug reports and feature requests
This repository accepts bug reports and feature requests for all Redux devices, not just for the vendordep. 

Remember to use the search feature to check if your issue or feature request has already been posted!

## Layout

Like most WPILib-compatible vendordeps, the build is split into 3 libraries: a Java, C++, and driver library. 

The Java and C++ libraries are open source (under src/main/java and src/main/native respectfully).

The ReduxCore driver library is closed source and is omitted from the public copy of this repo.

ReduxCore mostly does not handle device-level logic and exists to route device messages (typically CAN packets) between the higher-level libraries, device buses, and Alchemist.

## Architecture

The higher-level Java and C++ libraries spawn a thread (the CanandEventLoop) that continuously polls ReduxCore for incoming CAN messages.

The CanandEventLoop then runs a callback on subscribing device classes whose arbitration IDs match to asynchonously handle and parse the incoming message.

Device classes also send CAN messages through ReduxCore but ultimately assemble the message before passing it off. 

The vendordep was set up this way to maximize flexibility in implementation -- users may create their own device classes (by subclassing CanandDevice) and reimplement existing device classes with completely different API surfaces and message handling if they wish. 
Alternatively, they can subclass existing classes and override their handlers to function differently. 

Overall, we see value in users being able to see more "under the hood" and we hope to see what the community comes up with.

## Building 
The public copy of this repo is currently unable to be built as it lacks the (closed-source) driver. 

Ideally, we would pull the binary assets of the driver from Maven -- however, none of us at Redux Robotics actually understand GradleRIO well enough to make this work. 

We are accepting PRs that make the repo build, however.

Theoretically, if the build _were_ to work, it would look something like this (at least for OS X and Linux):

```bash
# Build the vendordep
./gradlew build

# Install the vendordep
./gradlew publishToMavenLocal && cp -r ${HOME}/.m2/repository/com/reduxrobotics ${HOME}/wpilib/${frcYear}/maven/com
```

## Contributing

We will accept PRs and general contributions. 
General guidelines for doing so will as a rule of thumb be similar to WPILib's [guidelines](https://github.com/wpilibsuite/allwpilib/blob/main/CONTRIBUTING.md) and [code of conduct](https://github.com/wpilibsuite/allwpilib/blob/main/CODE_OF_CONDUCT.md).


# Styleguide

4-spaces everywhere. 

## Java
use Java-standard `PascalCase` for classes/interfaces and `camelCase` for everything else.

Constants/enum variants should be `k`-prefixed (e.g. `kRangeMode`)

## C++
User-facing API methods and classes must use `PascalCase`.

Internal-facing methods may use `camelCase` if associated with a class or use `snake_case` for free-standing/module scope methods.

Constants/enum variants should be `k`-prefixed (e.g. `kRangeMode`)

Inline in headers short functions that do not pull in headers that you otherwise wouldn't expose to end users. 

### Considerations for Python bindings

Avoid using overloads in C++ in favor of optional arguments; Python bindings use the C++ api and Python does not have overloads.

Avoid using too complex templates as otherwise Python bindgen may choke on them.

## Rust
Avoid excessive scope nesting. 4-spaces everywhere.
