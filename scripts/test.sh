#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  expected_stdout="$3"

  cargo run --bin rust-9cc -- "$input" > tmp.s 2> /dev/null
  cc -o other.o -c clang/other.c
  cc -o tmp tmp.s other.o
  ./tmp > output.txt

  actual=$?

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
  if [ "$expected_stdout" != "" ]; then
    output=$(cat output.txt)
    if [ "$output" = "$expected_stdout" ]; then
      echo "$input =(stdout)> $output"
    else
      echo "$input =(stdout)> $expected_stdout expected, but got $output"
      exit 1
    fi
  fi
}

assert 20 "main() { a = 10; b = &a; *b = 20; return a; }"
assert 55 "fib(n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } main() { return fib(10); }"
assert 8 "fib(n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } main() { return fib(6); }"
assert 7 "hoge(a, b, c) { return a + b * c; } main() { return hoge(1, 2, 3); }"
assert 30 "hoge(a, b) { return a + b; } main() { return hoge(10, 20); }"
assert 2 "main() { return 1 + 1; }"
assert 0 "main() {zig(1, 3); return 0; }" "1 3"
assert 0 "main() {bar(1, 3); return 0; }" "4"
assert 0 "main() {foo(); return 0; }" "Hello World"
assert 2 "main() {{i = 0; i = i + 1; i = i + 1; return i;} }"
assert 55 "main() {foo = 0; i = 1; while(i <= 10) { foo = foo + i; i = i + 1; } return foo; }"
assert 55 "main() {foo = 0; i = 1; while(foo < 55) foo = foo + i; return foo; }"
assert 55 "main() {foo = 0; for (i = 1; i <= 10; i = i + 1) foo = foo + i; return foo; }"
assert 10 "main() {foo = 1; bar = 0; if (foo == 1)  bar = 10; return bar; }"
assert 0 "main() {foo = 0; bar = 0; if (foo == 1)  bar = 10; return bar; }"
assert 20 "main() {foo = 0; if (foo == 1)  return 10;  else  return 20;  }"
assert 10 "main() {foo = 1; if (foo == 1)  return 10;  else  return 20;  }"
assert 10 "main() {return 10; return 5; return 1 + 1; }"
assert 3 "main() {foo=1;bar=2;return foo+bar; }"
assert 3 "main() {foo=1;bar=2;foo+bar; }"
assert 0 "main() {0; }"
assert 42 "main() {42; }"
assert 21 "main() {5+20-4; }"
assert 41 "main() { 12 + 34 - 5 ; }"
assert 0 "main() { 1 + -1 ; }"
assert 51 "main() { 12 + 34 - -5  ; }"
assert 41 "main() { 12 + 34 - +5  ; }"
assert 2 "main() {1++1; }"
assert 0 "main() {1+-1; }"
assert 5 "main() { return 10 / 2; }"
assert 2 "main() { return 1 * 2; }"
assert 1 "main() {1 == 1; }"
assert 1 "main() {1 <= 1; }"
assert 1 "main() {1 >= 1; }"
assert 0 "main() {1 == 2; }"
assert 1 "main() {1 < 2; }"
assert 1 "main() {1 <= 2; }"
assert 0 "main() {1 > 2; }"
assert 0 "main() {1 >= 2; }"
assert 0 "main() {1 > 2; }"
assert 0 "main() {1 >= 2; }"
assert 22 "main() {a=1+1;a+20; }"


echo OK
