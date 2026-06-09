#!/bin/bash

LOG_FILE="/tmp/shell_debug.log"

echo "--- New Completion Request ---" >> "$LOG_FILE"
echo "arg[1] (Command):  '$1'" >> "$LOG_FILE"
echo "arg[2] (Current):  '$2'" >> "$LOG_FILE"
echo "arg[3] (Preceding): '$3'" >> "$LOG_FILE"
echo "COMP_LINE:         '${COMP_LINE}'" >> "$LOG_FILE"
echo "COMP_POINT:        '${COMP_POINT}'" >> "$LOG_FILE"

if [ "$1" = "gat" ]; then
    if [ "$2" = "ad" ]; then
        if [ -n "$COMP_LINE" ] && [ -n "$COMP_POINT" ]; then
            echo "add"
            exit 0
        else
            echo "ERROR: COMP_LINE or COMP_POINT missing!" >> "$LOG_FILE"
            exit 1
        fi
    fi

    if [ "$2" = "" ] && { [ "$3" = "" ] || [ "$3" = "gat" ]; }; then
        echo "add"
        echo "commit"
        echo "push"
        exit 0
    fi

    if [ "$2" = "c" ] || [ "$2" = "ch" ] || [ "$2" = "che" ]; then
        echo "checkout"
        echo "cherry-pick"
        exit 0
    fi

    if [ "$2" = "chec" ] || [ "$2" = "check" ] || [ "$2" = "checkou" ]; then
        echo "checkout"
        exit 0
    fi

    if [ "$2" = "set" ] && [ "$3" = "remote" ]; then
        echo "set-url"
        exit 0
    fi

fi