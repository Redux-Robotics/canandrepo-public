"""
This is useful in CI to make sure that actions/setup-rust-toolchain only sets up the targets you care about.

Usage:
python tools/filter_rust_toolchain.py
"""

if __name__ == "__main__":
    import sys
    with open("rust-toolchain.toml", "r") as f:
        data = f.read()
    
    targets = sys.argv[1:]

    in_targets = False
    for line in data.splitlines():
        if line.startswith("targets = ["):
            in_targets = True
        elif in_targets:
            if line.startswith("]"):
                in_targets = False
            else:
                found = False
                for target in targets:
                    if target in line:
                        found = True
                if not found:
                    continue
        print(line)