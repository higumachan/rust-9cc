#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  step="$3"

  cargo run --bin rust-9cc -- "$input" > tmp.s 2> /dev/null
  cc -o tmp tmp.s
  ./tmp

  actual=$?

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 10 "return 10; return 5; return 1 + 1;"
assert 3 "foo=1;bar=2;return foo+bar;"
assert 3 "foo=1;bar=2;foo+bar;"
assert 0 "0;" 01
assert 42 "42;" 01
assert 21 "5+20-4;" 02
assert 41 " 12 + 34 - 5 ;" 03
assert 0 " 1 + -1 ;" 03
assert 51 " 12 + 34 - -5  ;" 03
assert 41 " 12 + 34 - +5  ;" 03
assert 2 "1++1;" 03
assert 0 "1+-1;" 03
assert 1 "1 == 1;" 03
assert 1 "1 <= 1;" 03
assert 1 "1 >= 1;" 03
assert 0 "1 == 2;" 03
assert 1 "1 < 2;" 03
assert 1 "1 <= 2;" 03
assert 0 "1 > 2;" 03
assert 0 "1 >= 2;" 03
assert 0 "1 > 2;" 03
assert 0 "1 >= 2;" 03
assert 22 "a=1+1;a+20;"


echo OK
