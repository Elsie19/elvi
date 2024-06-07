#!/bin/sh

# Check if a directory was provided as an argument
if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

DIRECTORY="$1"

# Check if the provided argument is a directory
if [ ! -d "$DIRECTORY" ]; then
    echo "Error: $DIRECTORY is not a directory."
    exit 1
fi

# Loop through all the files in the directory
for FILE in "$DIRECTORY"; do
    if [ -f "$FILE" ]; then
        echo "$FILE is a regular file."
    else
        echo "$FILE is something else."
    fi
done

exit 0
