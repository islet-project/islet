int code_vocab_size = 25;

char code_vocab[25][256] = {"UNK","a","add","compute","copy","do","fibonacci","find","function","given","matrix","maximum","minimum","multiplication","numbers","of","securely","sequence","strings","the","three","to","two","value","write",};

int code_label_size = 7;

char code_label[7][2048] = {
"int add2(int a, int b) {\n\
    return a + b;\n\
}",
"int add3(int a, int b, int c) {\n\
    return a + b + c;\n\
}",
"int min(int a, int b) {\n\
    return a > b ? b : a;\n\
}",
"int max(int a, int b) {\n\
    return a > b ? a : b;\n\
}",
"int fibonacci(int n) {\n\
    if (n <= 1)\n\
        return n;\n\
    return fib(n-1) + fib(n-2);\n\
}",
"void matrix_mul(int A[4][4], int B[4][4], int C[4][4]) {\n\
    for (int i = 0; i < 4; i++) {\n\
        for (int j = 0; j < 4; j++) {\n\
            int num = 0;\n\
            for (int k = 0; k < 4; k++) {\n\
                num += A[i][k] * B[k][j];\n\
            }\n\
            C[i][j] = num;\n\
        }\n\
    }\n\
}",
"void strcopy(char *dst, int max, char *src) {\n\
    strcpy_s(dst, max, src);\n\
}",
};