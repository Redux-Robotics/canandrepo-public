#!/bin/bash
cargo run -p dbcgen messages dbc 0
python -m canandmessage_translingual.python messages
python -m canandmessage_translingual.java ../ReduxLib
python -m canandmessage_translingual.cpp ../ReduxLib