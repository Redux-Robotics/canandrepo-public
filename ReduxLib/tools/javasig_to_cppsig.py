import re
import sys

if __name__ == "__main__":
    with open(sys.argv[1]) as f:
        file = f.read()
    
    nl = []
    for line in file.splitlines():
        sline = line.strip()
        if sline.startswith("public ") and sline.endswith("{"):
            sline = sline.replace("boolean", "bool")
            parts = sline.split(" ")

            lpad = (len(line) - len(line.lstrip())) * " "

            cname = parts[2][0].upper() + parts[2][1:]
            nl.append(lpad + " ".join([parts[1], cname] + parts[3:-1]) + ";")
        else:
            nl.append(line)
    
    with open(sys.argv[1], "w") as f:
        f.write("\n".join(nl))
