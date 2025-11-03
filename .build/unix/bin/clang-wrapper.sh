#!/bin/sh
filtered_args=""
skip_next=0

for arg in "$@"; do
    if [ $skip_next -gt 0 ]; then
        skip_next=0
    else
        if [ "$arg" == "-L" ]; then
            skip_next=1
        else
            filtered_args="$filtered_args \"$arg\""
        fi
    fi
done

eval "set -- $filtered_args"
exec clang "$@"
