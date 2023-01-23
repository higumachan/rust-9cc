#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  step="$3"

  cargo run --bin step$3 -- "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 0 01
assert 42 42 01
assert 21 "5+20-4" 02

echo OK
