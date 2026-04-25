#!/usr/bin/env python3
import sys
import re

PATTERN = r'^(feat|fix|docs|refactor|test|chore|ci|perf|style|build)(\(.+\))?: .{1,100}'

def main():
    commit_msg_file = sys.argv[1]
    with open(commit_msg_file, 'r') as f:
        first_line = f.readline().strip()

    if not re.match(PATTERN, first_line):
        print(f"ERROR: Commit message does not follow conventional commits format.")
        print(f"  Got: '{first_line}'")
        print(f"  Expected: type(scope): description")
        print(f"  Types: feat, fix, docs, refactor, test, chore, ci, perf, style, build")
        sys.exit(1)

if __name__ == '__main__':
    main()
