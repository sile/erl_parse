%%%%
%%%% file: hello.erl
%%%%
-module(hello).

-export([world/0]).

%% @doc Prints "Hello World!" to the standard output.
-spec world() -> ok.
world() ->
    io:format("Hello World!"),
    ok.
