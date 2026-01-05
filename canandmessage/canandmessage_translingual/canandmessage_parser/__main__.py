from . import parse_spec_to_device, parse_spec
import sys

dev = parse_spec_to_device(sys.argv[1])
#dev = parse_spec(sys.argv[1])