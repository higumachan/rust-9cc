#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  expected_stdout="$3"
  my_arch=$(arch)

  cargo run --bin rust-9cc -- "$input" > tmp.s 2> /dev/null

  if [ "$my_arch" = "x86_64" ]; then

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
  fi
}

assert 4 "int main() {int *a; return sizeof(*a);}"
assert 8 "int main() {int *a; return sizeof(a);}"
assert 4 "int main() {int a; return sizeof(a);}"
assert 8 "int main() {int *p; alloc4(&p, 1, 2, 4, 8); int *q; q = p + 2; q = p + 3; return *q;}"
assert 4 "int main() {int *p; alloc4(&p, 1, 2, 4, 8); int *q; q = p + 2; return *q;}"
assert 55 "int main() {int foo; int i; foo = 0; i = 1; while(i <= 10) { foo = foo + i; i = i + 1; } return foo; }"
assert 45 "int f(int n) {int foo; int i; foo = 0; for (i = 1; i <= n; i = i + 1) foo = foo + i; return foo; } int main() { return f(9); }"
assert 0 "int fib(int n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } int main() { return fib(0); }"
assert 1 "int fib(int n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } int main() { return fib(1); }"
assert 1 "int fib(int n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } int main() { return fib(2); }"
assert 80 "int f2(int a) { return a * 2; } int f1(int a, int b) {  return f2(a) + b; } int main() { return f1(30, 20); }"
assert 10 "int hoge(int a, int b) {  if (a > b) { return 10; } return 30; } int main() { return hoge(30, 20); }"
assert 30 "int hoge(int a, int b) { return a + b; } int main() { return hoge(10, 20); }"
assert 100 "int f(int *a) { *a = 100; return 0; } int main() { int a; f(&a); return a; }"
assert 55 "int fib(int n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } int main() { return fib(10); }"
assert 100 "int f(int *a) { *a = 100; return 0; } int main() { int a; f(&a); return a; }"
assert 20 "int main() { int a; int *b; int **c; a = 10; c = &b; *c = &a; *b = a + **c; return a; }"
assert 10 "int main() { int a; int *b; int **c; a = 10; c = &b; *c = &a;  *b = 10; return a; }"
assert 30 "int main() { int a; int *b; int **c; a = 10; b = &a; c = &b; **c = 30; return a; }"
assert 20 "int main() { int a; int *b; a = 10; b = &a; *b = 20; return a; }"
assert 10 "int main() { int a; a = 10; return a; }"
assert 20 "int main() { int a; int* b; a = 10; b = &a; *b = 20; return a; }"
assert 8 "int fib(int n) { if (n == 0) { return 0; } if (n == 1) { return 1; } return fib(n - 1) + fib(n - 2); } int main() { return fib(6); }"
assert 7 "int hoge(int a, int b, int c) { return a + b * c; } int main() { return hoge(1, 2, 3); }"
assert 30 "int hoge(int a, int b) { return a + b; } int main() { return hoge(10, 20); }"
assert 2 "int main() { return 1 + 1; }"
assert 0 "int main() {zig(1, 3); return 0; }" "1 3"
assert 0 "int main() {bar(1, 3); return 0; }" "4"
assert 0 "int main() {foo(); return 0; }" "Hello World"
assert 2 "int main() {{int i; i = 0; i = i + 1; i = i + 1; return i;} }"
assert 55 "int main() {int foo; int i; foo = 0; i = 1; while(foo < 55) foo = foo + i; return foo; }"
assert 55 "int main() {int foo; int i; foo = 0; for (i = 1; i <= 10; i = i + 1) foo = foo + i; return foo; }"
assert 10 "int main() {int foo; int bar; foo = 1; bar = 0; if (foo == 1)  bar = 10; return bar; }"
assert 0 "int main() {int foo; int bar; foo = 0; bar = 0; if (foo == 1)  bar = 10; return bar; }"
assert 20 "int main() {int foo; foo = 0; if (foo == 1)  return 10;  else  return 20;  }"
assert 10 "int main() {int foo; foo = 1; if (foo == 1)  return 10;  else  return 20;  }"
assert 10 "int main() {return 10; return 5; return 1 + 1; }"
assert 3 "int main() {int foo; int bar; foo=1;bar=2;return foo+bar; }"
assert 3 "int main() {int foo; int bar; foo=1;bar=2;foo+bar; }"
assert 0 "int main() {0; }"
assert 42 "int main() {42; }"
assert 21 "int main() {5+20-4; }"
assert 41 "int main() { 12 + 34 - 5 ; }"
assert 0 "int main() { 1 + -1 ; }"
assert 51 "int main() { 12 + 34 - -5  ; }"
assert 41 "int main() { 12 + 34 - +5  ; }"
assert 2 "int main() {1++1; }"
assert 0 "int main() {1+-1; }"
assert 5 "int main() { return 10 / 2; }"
assert 2 "int main() { return 1 * 2; }"
assert 1 "int main() {1 == 1; }"
assert 1 "int main() {1 <= 1; }"
assert 1 "int main() {1 >= 1; }"
assert 0 "int main() {1 == 2; }"
assert 1 "int main() {1 < 2; }"
assert 1 "int main() {1 <= 2; }"
assert 0 "int main() {1 > 2; }"
assert 0 "int main() {1 >= 2; }"
assert 0 "int main() {1 > 2; }"
assert 0 "int main() {1 >= 2; }"
assert 22 "int main() {int a; a=1+1;a+20; }"

echo OK
