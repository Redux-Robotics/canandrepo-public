from typing import Type, Dict, Tuple, List, Any
from pathlib import Path
from enum import Enum
import json
import yaml
import os

from canandmessage_translingual.canandmessage_parser.model_impl import impl_DType_from_sig, impl_DType_from_type
from .canandmessage_parser import *
from .canandmessage_parser import toml_defs
from .canandmessage_parser import DTypeOnion

def format_dtype_for_openapi(dtype: str, dev: toml_defs.DeviceSpec) -> Dict[str, Any]:
    """Convert dtype to OpenAPI schema format"""
    if dtype.startswith("enum:"):
        return {
            "type": "string",
            "enum": [f"ENUM_{dtype[5:].upper()}"],
            "description": f"Enum type: {dtype[5:]}"
        }
    elif dtype.startswith("sint:"):
        bits = int(dtype[5:])
        return {
            "type": "integer",
            "format": f"int{bits}",
            "minimum": -(2**(bits-1)),
            "maximum": 2**(bits-1) - 1,
            "description": f"Signed {bits}-bit integer"
        }
    elif dtype.startswith("uint:"):
        bits = int(dtype[5:])
        return {
            "type": "integer",
            "format": f"uint{bits}",
            "minimum": 0,
            "maximum": 2**bits - 1,
            "description": f"Unsigned {bits}-bit integer"
        }
    elif dtype.startswith("buf:"):
        n_bytes = (int(dtype[4:]) + 1) // 8
        return {
            "type": "array",
            "items": {"type": "integer", "minimum": 0, "maximum": 255},
            "minItems": n_bytes,
            "maxItems": n_bytes,
            "description": f"Byte array of {n_bytes} bytes"
        }
    elif dtype.startswith("pad:"):
        return {
            "type": "integer",
            "minimum": 0,
            "maximum": 0,
            "description": f"Padding field ({dtype[4:]} bits)"
        }
    elif dtype == "bool" or dtype == "bit":
        return {
            "type": "boolean",
            "description": "Boolean value"
        }
    elif dtype.startswith("float:"):
        width = dtype[-2:]
        if width == "24":
            return {
                "type": "number",
                "format": "float",
                "description": "24-bit float"
            }
        elif width == "32":
            return {
                "type": "number",
                "format": "float",
                "description": "32-bit float"
            }
        else:
            return {
                "type": "number",
                "format": "double",
                "description": "64-bit double"
            }
    else:
        # Check if it's a custom type defined in the device specification
        if dtype in dev.types:
            type_spec = dev.types[dtype]
            # Convert the custom type to OpenAPI format
            if type_spec.btype == "sint":
                return {
                    "type": "integer",
                    "format": f"int{type_spec.bits}",
                    "minimum": -(2**(type_spec.bits-1)),
                    "maximum": 2**(type_spec.bits-1) - 1,
                    "description": type_spec.comment
                }
            elif type_spec.btype == "uint":
                return {
                    "type": "integer",
                    "format": f"uint{type_spec.bits}",
                    "minimum": 0,
                    "maximum": 2**type_spec.bits - 1,
                    "description": type_spec.comment
                }
            elif type_spec.btype == "float":
                return {
                    "type": "number",
                    "format": "float" if type_spec.bits <= 32 else "double",
                    "description": type_spec.comment
                }
            elif type_spec.btype == "bitset":
                return {
                    "type": "integer",
                    "format": f"uint{type_spec.bits}",
                    "minimum": 0,
                    "maximum": 2**type_spec.bits - 1,
                    "description": type_spec.comment
                }
            else:
                return {
                    "type": "string",
                    "description": f"Custom type: {dtype} ({type_spec.comment})"
                }
        else:
            return {
                "type": "string",
                "description": f"Custom type: {dtype}"
            }

def generate_message_schema(msg: toml_defs.DeviceMessageSpec, dev: toml_defs.DeviceSpec) -> Dict[str, Any]:
    """Generate OpenAPI schema for a message"""
    properties = {}
    required = []
    
    for signal in msg.signals:
        if not signal.optional:
            required.append(signal.name)
        
        properties[signal.name] = {
            **format_dtype_for_openapi(signal.dtype, dev),
            "description": signal.comment
        }
    
    return {
        "type": "object",
        "properties": properties,
        "required": required,
        "description": msg.comment
    }

def generate_openapi_spec(dev: toml_defs.DeviceSpec) -> Dict[str, Any]:
    """Generate complete OpenAPI specification for a device"""
    
    # Get device messages that are public and from device to host
    device_messages = [
        (name, msg) for name, msg in dev.msg.items() 
        if msg.is_public and msg.source in ["device", "both"]
    ]
    
    # Generate message schemas
    message_schemas = {}
    message_endpoints = {}
    
    for name, msg in device_messages:
        schema_name = f"{name}Message"
        message_schemas[schema_name] = generate_message_schema(msg, dev)
        
        # Convert message name to camelCase for endpoint
        camel_name = ''.join(word.capitalize() for word in name.split('_'))
        
        # Create endpoint for this message
        endpoint_path = f"/{dev.name.lower()}/{{bus_id}}/{{session_id}}/{camel_name}"
        message_endpoints[endpoint_path] = {
            "get": {
                "summary": f"Get {name} message",
                "description": msg.comment,
                "parameters": [
                    {
                        "name": "bus_id",
                        "in": "path",
                        "required": True,
                        "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                        "description": "CAN bus ID (hexadecimal)"
                    },
                    {
                        "name": "session_id", 
                        "in": "path",
                        "required": True,
                        "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                        "description": "Session ID (hexadecimal)"
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Success",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": f"#/components/schemas/{schema_name}"}
                            }
                        }
                    },
                    "404": {
                        "description": "Not found"
                    }
                }
            }
        }
    
    # Base OpenAPI specification
    openapi_spec = {
        "openapi": "3.0.3",
        "info": {
            "title": f"{dev.name} CAN API",
            "description": f"REST API for {dev.name} CAN device communication",
            "version": "1.0.0"
        },
        "servers": [
            {
                "url": "http://localhost:7244",
                "description": "Development server"
            }
        ],
        "paths": {
            "/": {
                "get": {
                    "summary": "Root/Banner",
                    "description": "Returns HTML banner with service info and version",
                    "responses": {
                        "200": {
                            "description": "HTML banner",
                            "content": {
                                "text/html": {
                                    "schema": {"type": "string"}
                                }
                            }
                        }
                    }
                }
            },
            "/version": {
                "get": {
                    "summary": "Version Info",
                    "description": "Returns current package version string",
                    "responses": {
                        "200": {
                            "description": "Version string",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "version": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/buses/open": {
                "post": {
                    "summary": "Open Bus",
                    "description": "Opens a CAN bus connection",
                    "parameters": [
                        {
                            "name": "params",
                            "in": "query",
                            "required": True,
                            "schema": {"type": "string"},
                            "description": "Bus parameters/name"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Bus opened successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id": {"type": "integer"},
                                            "params": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/sessions/open/{bus_id}/{filter_id}/{filter_mask}": {
                "post": {
                    "summary": "Open Session",
                    "description": "Creates a CAN session with filtering",
                    "parameters": [
                        {
                            "name": "bus_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Bus ID (hexadecimal)"
                        },
                        {
                            "name": "filter_id",
                            "in": "path", 
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Filter ID (hexadecimal)"
                        },
                        {
                            "name": "filter_mask",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Filter mask (hexadecimal)"
                        },
                        {
                            "name": "X-Arbitration",
                            "in": "header",
                            "required": False,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}$"},
                            "description": "Device arbitration ID (6-byte array in format XX-XX-XX-XX-XX-XX)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Session opened successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id": {"type": "integer"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/sessions/{bus_id}/{session_id}/enumerate": {
                "get": {
                    "summary": "Enumerate Bus",
                    "description": "Discovers devices on the CAN bus",
                    "parameters": [
                        {
                            "name": "bus_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Bus ID (hexadecimal)"
                        },
                        {
                            "name": "session_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Session ID (hexadecimal)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Device enumeration results",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "known_ids": {
                                                "type": "object",
                                                "additionalProperties": {
                                                    "type": "array",
                                                    "items": {"type": "integer", "minimum": 0, "maximum": 255},
                                                    "minItems": 6,
                                                    "maxItems": 6,
                                                    "description": "6-byte serial number"
                                                },
                                                "description": "CAN ID to serial number mapping"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/sessions/{bus_id}/{session_id}/conflicts": {
                "get": {
                    "summary": "Get Conflicts",
                    "description": "Returns CAN ID conflicts on the bus",
                    "parameters": [
                        {
                            "name": "bus_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Bus ID (hexadecimal)"
                        },
                        {
                            "name": "session_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Session ID (hexadecimal)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Conflict information",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "conflicts": {
                                                "type": "object",
                                                "additionalProperties": {
                                                    "type": "array",
                                                    "items": {
                                                        "type": "array",
                                                        "items": {"type": "integer", "minimum": 0, "maximum": 255},
                                                        "minItems": 6,
                                                        "maxItems": 6,
                                                        "description": "6-byte serial number"
                                                    }
                                                },
                                                "description": "CAN ID to array of conflicting serial numbers"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/devices/{bus_id}/{session_id}/{device_id}/settings": {
                "get": {
                    "summary": "Get Device Settings",
                    "description": "Retrieves cached device settings",
                    "parameters": [
                        {
                            "name": "bus_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Bus ID (hexadecimal)"
                        },
                        {
                            "name": "session_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Session ID (hexadecimal)"
                        },
                        {
                            "name": "device_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Device ID (hexadecimal)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Device settings",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "setting": {"type": "object"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/devices/{bus_id}/{session_id}/{device_id}/name": {
                "get": {
                    "summary": "Get Device Name",
                    "description": "Gets device name information",
                    "parameters": [
                        {
                            "name": "bus_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Bus ID (hexadecimal)"
                        },
                        {
                            "name": "session_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Session ID (hexadecimal)"
                        },
                        {
                            "name": "device_id",
                            "in": "path",
                            "required": True,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]+$"},
                            "description": "Device ID (hexadecimal)"
                        },
                        {
                            "name": "X-Arbitration",
                            "in": "header",
                            "required": False,
                            "schema": {"type": "string", "pattern": "^[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}-[0-9a-fA-F]{2}$"},
                            "description": "Device arbitration ID (6-byte array in format XX-XX-XX-XX-XX-XX)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Device name information",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "name": {
                                                "type": "object",
                                                "additionalProperties": {
                                                    "type": "array",
                                                    "items": {"type": "integer", "minimum": 0, "maximum": 255},
                                                    "minItems": 6,
                                                    "maxItems": 6,
                                                    "description": "6-byte name setting"
                                                },
                                                "description": "Setting ID to name data mapping"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": message_schemas,
            "securitySchemes": {
                "CORS": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "Access-Control-Allow-Origin"
                }
            }
        },
        "security": [],
        "tags": [
            {
                "name": "General",
                "description": "General API endpoints"
            },
            {
                "name": "Bus Management", 
                "description": "CAN bus connection management"
            },
            {
                "name": "Session Management",
                "description": "CAN session management and device discovery"
            },
            {
                "name": "Device Management",
                "description": "Device-specific operations"
            },
            {
                "name": f"{dev.name} Messages",
                "description": f"Message endpoints for {dev.name} device"
            }
        ]
    }
    
    # Add device-specific message endpoints
    openapi_spec["paths"].update(message_endpoints)
    
    return openapi_spec

def gen_openapi_spec(dev: toml_defs.DeviceSpec, file: Path):
    """Generate OpenAPI specification for a device"""
    spec = generate_openapi_spec(dev)
    
    # Write to target directory
    Path("target/openapi").mkdir(parents=True, exist_ok=True)
    with open(f"target/openapi/{dev.name.lower()}.yaml", "w") as f:
        yaml.dump(spec, f, default_flow_style=False, sort_keys=False)
    
    return spec

if __name__ == "__main__":
    import sys
    from pathlib import Path
    
    if len(sys.argv) < 2:
        print("Usage: python openapi.py <device_toml_file>")
        sys.exit(1)
    
    file_path = Path(sys.argv[1])
    dev = parse_spec(file_path)
    spec = gen_openapi_spec(dev, file_path)
    
    print(f"Generated OpenAPI spec for {dev.name}")
    print(f"Spec written to: target/openapi/{dev.name.lower()}.yaml")
