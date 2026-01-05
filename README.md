# canandrepo-public

This is a monorepo for Redux Robotics open-source code. 

## License

* Contents of `crates/`, `canandmessage/` and `reduxfifo/xtask` are `MIT OR Apache-2.0`
* Contents of `reduxfifo/` are `LGPL-3.0-only` EXCEPT `reduxfifo/xtask` which is `MIT OR Apache-2.0`
* Contents of `ReduxLib/` are generally `MPL 2.0`

## Install guide

You need:

* rustup to install toolchains from https://rustup.rs/


## Building reduxlib 

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

