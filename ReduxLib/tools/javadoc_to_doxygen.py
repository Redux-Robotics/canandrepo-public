import re
import sys

link_reg = re.compile(r"{@link (.*?)}")
if __name__ == "__main__":
    with open(sys.argv[1]) as f:
        file = f.read()
    
    fnew = file
    for fmatch in link_reg.findall(file):
        fmatch_new = fmatch
        fparts = fmatch.split("#")
        if len(fparts) > 1 and fparts[1]:
            fmatch_new = fparts[0] + "::" + fparts[1][0].upper() + fparts[1][1:]
        fnew = fnew.replace("{@link " + fmatch + "}", fmatch_new)
        print("{@liink " + fmatch + "}", "->", fmatch_new)
    
    with open(sys.argv[1], "w") as f:
        f.write(fnew)
