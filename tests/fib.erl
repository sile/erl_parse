-module(fib).

-export([fib/1]).

-spec fib(non_neg_integer()) -> non_neg_integer().
fib(0) -> 0;
fib(1) -> 1;
fib(N) -> fib(N - 2) + fib(N - 1).
