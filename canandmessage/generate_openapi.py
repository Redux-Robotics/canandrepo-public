#!/usr/bin/env python3
"""
Generate OpenAPI specifications for all devices in the messages/ directory.
"""

import sys
from pathlib import Path
from canandmessage_translingual.openapi import gen_openapi_spec
from canandmessage_translingual.canandmessage_parser import parse_spec

def main():
    messages_dir = Path("messages")
    if not messages_dir.exists():
        print("Error: messages/ directory not found")
        sys.exit(1)
    
    # Find all TOML files in messages directory
    toml_files = list(messages_dir.glob("*.toml"))
    
    if not toml_files:
        print("No TOML files found in messages/ directory")
        sys.exit(1)
    
    print(f"Found {len(toml_files)} device specifications:")
    
    for toml_file in toml_files:
        print(f"  - {toml_file.name}")
    
    print("\nGenerating OpenAPI specifications...")
    
    for toml_file in toml_files:
        try:
            print(f"Processing {toml_file.name}...")
            dev = parse_spec(toml_file)
            gen_openapi_spec(dev, toml_file)
            print(f"  ✓ Generated {dev.name.lower()}.yaml")
        except Exception as e:
            print(f"  ✗ Error processing {toml_file.name}: {e}")
    
    print(f"\nOpenAPI specifications written to target/openapi/")
    print("You can now use these YAML files with OpenAPI tools like Swagger UI or Postman.")

if __name__ == "__main__":
    main()
