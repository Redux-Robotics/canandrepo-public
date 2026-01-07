# canandrepo-public

This is a monorepo for Redux Robotics open-source code. 

## Licenses

This repository uses multiple licenses.

If you wish to use code in this repository in a way that may be incompatible with its listed license, contact us at `support@reduxrobotics.com` and we can work something out.

### MIT OR Apache-2.0

* `crates/*`
* `canandmessage/*`
* `reduxfifo/xtask/*`
* `tools/*`
* `.gitignore`
* `.github/*`
* `.vscode/*`

### LGPL-3.0-only

* `reduxfifo/*` EXCEPT `reduxfifo/xtask/`

### Mozilla Public License 2.0

* `ReduxLib/*`

## Build guide (for advanced users)

To install ReduxLib, consult https://docs.reduxrobotics.com.

To build things in this repository, you need:

* rustup to install toolchains from https://rustup.rs/
* a copy of LLVM installed (e.g. `libclang-dev` from Debian) to build 
* the current version of WPILib
* A WPILib-compatible JVM (one comes for free in your WPILib install under `wpilib/YEAR/jdk` that you can set as your `JAVA_HOME`)


### Building reduxlib 

First build reduxfifo

```shell
$ cd reduxfifo
$ cargo xtask --all headers linuxathena auto
```

Then build ReduxLib and create a JSON that adds the build directories as Maven repos

```shell
$ cd ReduxLib
$ ./gradlew publish -PreleaseMode 
$ python3 mkvdepjson.py local > /path/to/your/robot/project/vendordeps/ReduxLib_Local.json
```

