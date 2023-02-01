//
// Created by Yuta Hinokuma on 2023/01/28.
//

#include <stdio.h>
#include <stdlib.h>

int foo() {
    printf("Hello World\n");
}

int bar(int a, int b) {
    printf("%d\n", a + b);
}

int zig(int a, int b) {
    printf("%d %d\n", a, b);
}

int alloc4(int **p, int a, int b, int c, int d) {
    *p = (int *)malloc(sizeof(int) * 4);
    (*p)[0] = a;
    (*p)[1] = b;
    (*p)[2] = c;
    (*p)[3] = d;
}
