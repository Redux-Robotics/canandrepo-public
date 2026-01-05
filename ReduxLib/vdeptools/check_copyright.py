"""
Checks that the copyright header is on every source file and yells at you if it isn't.
"""
import sys
from pathlib import Path
import itertools
copyright = """
// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.
""".strip()

if __name__ == "__main__":
    ret = 0
    root = Path(sys.argv[1])
    for p in itertools.chain(root.glob("src/main/java/**/*.java"),
                             root.glob("src/main/native/**/*.cpp"),
                             root.glob("src/main/native/**/*.h")):
        with open(p, "r") as f:
            file_data = f.read()
            if not file_data.startswith(copyright):
                print(p, "lacks a copyright header!!!")
                ret = 1
    sys.exit(ret)
