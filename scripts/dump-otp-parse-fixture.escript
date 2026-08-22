#!/usr/bin/env escript
%%! -noshell -noinput

%% Dump OTP erl_parse results as JSONL for `otp_diff`.
%%
%% Records the OTP tag from the `OTP_TAG` environment variable into the
%% fixture `_meta` object. The dump does not know which checkout it was
%% pointed at; the caller must set `OTP_TAG` to match `<OTP_ROOT>`.
%%
%% Each record is one JSON object. Stage fields (`tokenize`, `preprocess`,
%% `parse`) are `"ok"` or `"err"`. Snippets without macros/includes are
%% scanned and parsed directly (equivalent to `epp` as a no-op). OTP
%% source files go through `epp:open` / `epp:parse_erl_form`.
%%
%% Usage:
%%   OTP_TAG=OTP-29.0.5 escript scripts/dump-otp-parse-fixture.escript <OTP_ROOT>

-mode(compile).

main([Root]) ->
    case os:getenv("OTP_TAG") of
        false ->
            io:format(standard_error,
                      "OTP_TAG is not set (e.g. OTP_TAG=OTP-29.0.5)~n", []),
            halt(1);
        "" ->
            io:format(standard_error, "OTP_TAG is empty~n", []),
            halt(1);
        Tag ->
            emit_meta(Tag),
            lists:foreach(fun emit_synthetic/1, synthetics()),
            Files = lists:sort(walk(Root)),
            lists:foreach(fun(F) -> dump_otp_file(Root, F) end, Files)
    end;
main(_) ->
    io:format(standard_error,
              "usage: OTP_TAG=<tag> dump-otp-parse-fixture.escript <OTP_ROOT>~n", []),
    halt(1).

emit_meta(Tag) ->
    {{Y, M, D}, _} = calendar:universal_time(),
    Date = io_lib:format("~4..0B-~2..0B-~2..0B", [Y, M, D]),
    io:put_chars([${,
                  "\"_meta\":true,",
                  "\"otp_tag\":\"", escape(Tag), "\",",
                  "\"generated\":\"", Date, $",
                  $}, $\n]).

walk(Root) ->
    filelib:fold_files(Root, ".*\\.(erl|hrl)$", true,
                       fun(F, Acc) ->
                               case is_target(F) andalso not is_skipped(F) of
                                   true -> [F | Acc];
                                   false -> Acc
                               end
                       end, []).

is_target(Path) ->
    (string:find(Path, "/lib/") =/= nomatch
     andalso (string:find(Path, "/src/") =/= nomatch
              orelse string:find(Path, "/include/") =/= nomatch)
     andalso string:find(Path, "/test/") =:= nomatch)
        orelse string:find(Path, "/erts/preloaded/src/") =/= nomatch.

is_skipped(Path) ->
    string:find(Path, "/lib/wx/api_gen/wx_extra/") =/= nomatch
        orelse lists:suffix("/lib/kernel/test/socket_sctp_SUITE.erl", Path)
        orelse string:find(Path, "/lib/wx/") =/= nomatch
        orelse string:find(Path, "/lib/snmp/") =/= nomatch
        orelse string:find(Path, "/lib/public_key/src/") =/= nomatch
        orelse string:find(Path, "/lib/eldap/src/") =/= nomatch
        orelse lists:any(fun(Suf) -> lists:suffix(Suf, Path) end,
                         ["/lib/compiler/src/beam_ssa_alias.erl",
                          "/lib/compiler/src/beam_ssa_alias_debug.hrl",
                          "/lib/compiler/src/beam_ssa_ss.erl",
                          "/lib/stdlib/src/graph.erl",
                          "/lib/stdlib/src/peer.erl",
                          "/lib/compiler/src/beam_asm.erl",
                          "/lib/compiler/src/beam_disasm.erl",
                          "/lib/kernel/src/inet_dns.erl",
                          "/lib/common_test/src/ct_snmp.erl",
                          "/lib/stdlib/src/io_ansi.erl",
                          "/lib/syntax_tools/src/merl_tests.erl"]).

dump_otp_file(Root, File) ->
    case read_utf8(File) of
        {ok, Src} ->
            Tok = tokenize_stage(Src),
            case Tok of
                err ->
                    emit_obj([{"id", id_otp(File)},
                              {"kind", "otp_file"},
                              {"mode", "module"},
                              {"path", File},
                              {"tokenize", "err"},
                              {"preprocess", "err"},
                              {"parse", "err"}]);
                ok ->
                    Includes = includes(Root, File),
                    case epp:open([{name, File}, {includes, Includes},
                                   {default_encoding, utf8}]) of
                        {ok, Epp} ->
                            Forms = collect_forms(Epp, File),
                            epp:close(Epp),
                            emit_otp_ok(File, Forms);
                        {error, _} ->
                            emit_obj([{"id", id_otp(File)},
                                      {"kind", "otp_file"},
                                      {"mode", "module"},
                                      {"path", File},
                                      {"tokenize", "ok"},
                                      {"preprocess", "err"},
                                      {"parse", "err"}])
                    end
            end;
        _ ->
            ok
    end.

emit_otp_ok(File, Forms) ->
    Parse = case lists:all(fun(#{parse := ok}) -> true; (_) -> false end, Forms) of
                true -> "ok";
                false -> "err"
            end,
    FormJson = string:join([form_object(F) || F <- Forms], ","),
    io:put_chars([${,
                  "\"id\":\"", escape(id_otp(File)), "\",",
                  "\"kind\":\"otp_file\",",
                  "\"mode\":\"module\",",
                  "\"path\":\"", escape(File), "\",",
                  "\"tokenize\":\"ok\",",
                  "\"preprocess\":\"ok\",",
                  "\"parse\":\"", Parse, "\",",
                  "\"forms\":[", FormJson, $],
                  $}, $\n]).

collect_forms(Epp, SourceFile) ->
    case epp:parse_erl_form(Epp) of
        {ok, Form} ->
            [abs_form(Form, SourceFile) | collect_forms(Epp, SourceFile)];
        {error, ErrorInfo} ->
            [error_form(ErrorInfo) | collect_forms(Epp, SourceFile)];
        {eof, _} ->
            []
    end.

abs_form({attribute, Anno, file, {Path, _Line}}, _SourceFile) ->
    case is_epp_file_path(Path) of
        true ->
            #{parse => ok, category => "attribute", line => anno_line(Anno),
              epp => file};
        false ->
            #{parse => ok, category => "attribute", line => anno_line(Anno)}
    end;
abs_form({attribute, Anno, _, _}, _SourceFile) ->
    #{parse => ok, category => "attribute", line => anno_line(Anno)};
abs_form({function, Anno, _, _, _}, _SourceFile) ->
    #{parse => ok, category => "function", line => anno_line(Anno)};
abs_form(_, _SourceFile) ->
    #{parse => ok, category => "other", line => 1}.

%% OTP `epp` file markers use a filesystem path (`dir/name`); generated
%% `-file("yeccgramm.yrl", 0).` markers use a basename only.
is_epp_file_path(Path) when is_binary(Path) ->
    is_epp_file_path(binary_to_list(Path));
is_epp_file_path(Path) when is_list(Path) ->
    filename:dirname(Path) =/= ".";
is_epp_file_path(_) ->
    false.

error_form({Loc, _, _}) ->
    #{parse => err, category => "error", line => loc_line(Loc)};
error_form(Other) ->
    #{parse => err, category => "error", line => loc_line(Other)}.

includes(Root, File) ->
    Globals = filelib:wildcard(filename:join([Root, "lib", "*", "include"])),
    build_include_paths(File, Root, Globals).

%% Mirrors `otp_conformance::build_include_paths` (empty CLI `-I`).
build_include_paths(Target, Root, Globals) ->
    Acc0 = add_parent_include_dirs(Target, []),
    Acc1 = add_app_src_walk_dirs(Target, Acc0),
    Acc2 = add_preloaded_kernel_src(Target, Root, Acc1),
    lists:foldl(fun add_unique/2, Acc2, Globals).

add_parent_include_dirs(Target, Acc) ->
    Dir = filename:dirname(Target),
    Acc1 = add_unique(Dir, Acc),
    Parent = filename:dirname(Dir),
    Include = filename:join(Parent, "include"),
    case Include =/= Dir of
        true -> add_unique(Include, Acc1);
        false -> Acc1
    end.

add_app_src_walk_dirs(Target, Acc) ->
    case find_app_src_dir(Target) of
        undefined -> Acc;
        AppSrc -> lists:foldl(fun add_unique/2, Acc, walk_dirs(AppSrc))
    end.

find_app_src_dir(Target) ->
    find_app_src_dir_up(filename:dirname(Target)).

find_app_src_dir_up(Dir) ->
    case filename:basename(Dir) of
        "src" -> Dir;
        _ ->
            Parent = filename:dirname(Dir),
            case Parent =:= Dir of
                true -> undefined;
                false -> find_app_src_dir_up(Parent)
            end
    end.

walk_dirs(Root) ->
    walk_dirs(Root, []).

walk_dirs(Root, Acc) ->
    case filelib:is_dir(Root) of
        false ->
            Acc;
        true ->
            Acc1 = add_unique(Root, Acc),
            lists:foldl(fun(Child, A) ->
                                case filelib:is_dir(Child) of
                                    true -> walk_dirs(Child, A);
                                    false -> A
                                end
                        end,
                        Acc1,
                        filelib:wildcard(filename:join(Root, "*")))
    end.

add_preloaded_kernel_src(Target, Root, Acc) ->
    case string:find(Target, "/erts/preloaded/src/") of
        nomatch ->
            Acc;
        _ ->
            KernelSrc = filename:join([Root, "lib", "kernel", "src"]),
            case filelib:is_dir(KernelSrc) of
                true -> add_unique(KernelSrc, Acc);
                false -> Acc
            end
    end.

add_unique(Path, Acc) ->
    case lists:member(Path, Acc) of
        true -> Acc;
        false -> [Path | Acc]
    end.

id_otp(File) ->
    "otp:" ++ File.

synthetics() ->
    [
     syn("syn:prec:mul-add", "expression", "1 + 2 * 3.", tree),
     syn("syn:prec:paren-mul", "expression", "(1 + 2) * 3.", tree),
     syn("syn:prec:match-right", "expression", "A = B = C.", tree),
     syn("syn:prec:andalso-orelse", "expression", "X orelse Y andalso Z.", tree),
     syn("syn:prec:catch-send", "expression", "catch A ! B.", tree),
     syn("syn:expr:call", "expression", "foo(1, 2).", parse),
     syn("syn:expr:remote", "expression", "m:f(1).", parse),
     syn("syn:expr:fun", "expression", "fun() -> ok end.", parse),
     syn("syn:expr:case", "expression", "case X of Y -> ok end.", parse),
     syn("syn:expr:if", "expression", "if true -> ok end.", parse),
     syn("syn:expr:receive", "expression", "receive X -> ok end.", parse),
     syn("syn:expr:try", "expression", "try 1 of 1 -> ok catch _ -> e after ok end.", parse),
     syn("syn:expr:maybe", "expression", "maybe X ?= 1 else _ -> 0 end.", parse),
     syn("syn:expr:begin", "expression", "begin 1, 2 end.", parse),
     syn("syn:expr:list", "expression", "[1, 2, 3].", parse),
     syn("syn:expr:tuple", "expression", "{1, 2}.", parse),
     syn("syn:expr:map", "expression", "#{a => 1}.", parse),
     syn("syn:expr:record", "expression", "#r{a = 1}.", parse),
     syn("syn:expr:bitstring", "expression", "<<1, 2>>.", parse),
     syn("syn:expr:comprehension", "expression", "[X || X <- [1]].", parse),
     syn("syn:otp29:call-chain", "expression", "f()().", parse),
     syn("syn:otp29:mc-multi", "expression", "[X, Y || X <- [1], Y <- [2]].", parse),
     syn("syn:otp29:native-record", "expression", "#mod:name{a = 1}.", parse),
     syn("syn:otp29:anon-record", "expression", "#_{a = 1}.", parse),
     syn("syn:fun:cons-pat", "module", "f([H|T]) -> ok.", parse),
     syn("syn:fun:guard-seq", "module", "f(X) when X > 0, is_atom(X) -> ok.", parse),
     syn("syn:attr:spec-union", "module", "-spec f() -> integer() | atom().", parse),
     syn("syn:attr:spec-range", "module", "-spec f() -> 1..10.", parse),
     syn("syn:attr:spec-map", "module", "-spec f() -> #{a => integer()}.", parse),
     syn("syn:attr:spec-record", "module", "-spec f() -> #r{a :: integer()}.", parse),
     syn("syn:attr:spec-fun", "module", "-spec f() -> fun((integer()) -> integer()).", parse),
     syn("syn:term:tuple", "term_list", "{ok, 1}.", parse),
     syn("syn:term:reject-var", "term_list", "X.", parse),
     syn("syn:term:reject-call", "term_list", "foo(1).", parse),
     syn("syn:term-list:seq", "term_list", "1. 2. 3.", parse),
     syn("syn:term-list:config", "term_list", "[{a, 1}].", parse),
     syn("syn:expr-mode:single", "expression", "1 + 2.", parse),
     syn("syn:expr-mode:comma-seq", "expression", "1, 2.", parse),
     syn("syn:attr:module", "module", "-module(m).", parse),
     syn("syn:attr:export", "module", "-export([f/0]).", parse),
     syn("syn:attr:spec", "module", "-spec f() -> ok.", parse),
     syn("syn:attr:callback", "module", "-callback f() -> ok.", parse),
     syn("syn:attr:type", "module", "-type t() :: ok.", parse),
     syn("syn:attr:opaque", "module", "-opaque t() :: ok.", parse),
     syn("syn:attr:record", "module", "-record(r, {a}).", parse),
     syn("syn:attr:unknown", "module", "-unknown(foo).", parse),
     syn("syn:fun:single", "module", "f() -> ok.", parse),
     syn("syn:fun:multi-clause", "module", "f(1) -> a; f(2) -> b.", parse),
     syn("syn:fun:module-and-fun", "module", "-module(m).\nf() -> ok.", parse),
     syn("syn:comment", "module", "f() -> % c\n ok.", parse),
     syn("syn:err:missing-dot", "module", "-module(m)\n", parse),
     syn("syn:err:bad-clause", "module", "f( -> ok.", parse),
     rec("syn:recovery:after-bad-form", "module", "1 2 3.\n-ok.", 1)
    ].

syn(Id, Mode, Source, Kind) ->
    #{id => Id, kind => "synthetic", mode => Mode, source => Source, extra => Kind}.

rec(Id, Mode, Source, Later) ->
    #{id => Id, kind => "recovery", mode => Mode, source => Source,
      extra => {recovery, Later}}.

emit_synthetic(#{id := Id, kind := Kind, mode := Mode, source := Source, extra := Extra}) ->
    {Tok, Pp, Parse, Forms, Tree} = run_snippet(Mode, Source, Extra),
    Fields = [{"id", Id},
              {"kind", Kind},
              {"mode", Mode},
              {"source", Source},
              {"tokenize", atom_to_list(Tok)},
              {"preprocess", atom_to_list(Pp)},
              {"parse", atom_to_list(Parse)}],
    Fields1 = case Forms of
                  [] -> Fields;
                  _ -> Fields ++ [{"forms_raw", Forms}]
              end,
    Fields2 = case Extra of
                  tree when Tree =/= undefined ->
                      Fields1 ++ [{"tree_raw", Tree}];
                  {recovery, Later} ->
                      Fields1 ++ [{"expect_later_forms", Later}];
                  _ ->
                      Fields1
              end,
    emit_obj(Fields2).

run_snippet(Mode, Source, Extra) ->
    case tokenize_stage(Source) of
        err -> {err, err, err, [], undefined};
        ok ->
            %% No macros/includes in synthetics, so `epp` would be a no-op.
            {Parse, Forms, Tree} = parse_snippet(Mode, Source, Extra),
            {ok, ok, Parse, Forms, Tree}
    end.

tokenize_stage(Src) ->
    case erl_scan:string(Src, {1, 1}, [text]) of
        {ok, _, _} -> ok;
        {error, _, _} -> err
    end.

parse_snippet("expression", Source, Extra) ->
    case erl_scan:string(Source, {1, 1}, []) of
        {ok, Ts, _} ->
            case erl_parse:parse_exprs(Ts) of
                {ok, [E]} ->
                    Tree = case Extra of tree -> shape(E); _ -> undefined end,
                    {ok, [], Tree};
                {ok, _} ->
                    {ok, [], undefined};
                {error, _Info} ->
                    {err, [], undefined}
            end;
        {error, _, _} ->
            {err, [], undefined}
    end;
parse_snippet("term_list", Source, _) ->
    case erl_scan:string(Source, {1, 1}, []) of
        {ok, Ts, _} ->
            {Parse, Forms} = parse_terms(Ts, []),
            {Parse, Forms, undefined};
        {error, _, _} ->
            {err, [], undefined}
    end;
parse_snippet("module", Source, _) ->
    case erl_scan:string(Source, {1, 1}, []) of
        {ok, Ts, _} ->
            {Parse, Forms} = parse_forms(Ts, []),
            {Parse, Forms, undefined};
        {error, _, _} ->
            {err, [], undefined}
    end.

parse_terms([], Acc) ->
    Parse = case lists:all(fun(#{parse := ok}) -> true; (_) -> false end, lists:reverse(Acc)) of
                true -> ok;
                false -> err
            end,
    {Parse, lists:reverse(Acc)};
parse_terms(Ts, Acc) ->
    case split_dot(Ts) of
        none -> parse_terms([], Acc);
        {FormToks, Rest} ->
            Rec = case erl_parse:parse_term(FormToks) of
                      {ok, _} -> #{parse => ok, category => "term", line => 1};
                      {error, Info} ->
                          #{parse => err, category => "error", line => error_line(Info)}
                  end,
            parse_terms(Rest, [Rec | Acc])
    end.

parse_forms([], Acc) ->
    Parse = case Acc =:= [] of
                true -> err;
                false ->
                    case lists:all(fun(#{parse := ok}) -> true; (_) -> false end,
                                   lists:reverse(Acc)) of
                        true -> ok;
                        false -> err
                    end
            end,
    {Parse, lists:reverse(Acc)};
parse_forms(Ts, Acc) ->
    case split_dot(Ts) of
        none -> parse_forms([], Acc);
        {FormToks, Rest} ->
            Rec = case erl_parse:parse_form(FormToks) of
                      {ok, Abs} -> abs_form(Abs, "<synthetic>");
                      {error, Info} -> error_form(Info)
                  end,
            parse_forms(Rest, [Rec | Acc])
    end.

split_dot([]) ->
    none;
split_dot(Ts) ->
    {Pre, Post} = lists:splitwith(fun(T) -> element(1, T) =/= dot end, Ts),
    case Post of
        [Dot | Rest] -> {Pre ++ [Dot], Rest};
        [] -> {Pre, []}
    end.

shape({op, _, Op, A, B}) ->
    [$[, $", "binop", $", $,, $", escape(atom_to_list(Op)), $", $,, shape(A), $,, shape(B), $]];
shape({op, _, Op, A}) ->
    [$[, $", "unary", $", $,, $", escape(atom_to_list(Op)), $", $,, shape(A), $]];
shape({match, _, A, B}) ->
    [$[, $", "binop", $", $,, $", "=", $", $,, shape(A), $,, shape(B), $]];
shape({maybe_match, _, A, B}) ->
    [$[, $", "binop", $", $,, $", "?=", $", $,, shape(A), $,, shape(B), $]];
shape({'catch', _, A}) ->
    [$[, $", "unary", $", $,, $", "catch", $", $,, shape(A), $]];
shape({integer, _, _}) -> "[\"integer\"]";
shape({float, _, _}) -> "[\"float\"]";
shape({atom, _, _}) -> "[\"atom\"]";
shape({var, _, _}) -> "[\"var\"]";
shape({char, _, _}) -> "[\"char\"]";
shape(Other) ->
    [$[, $", escape(io_lib:format("~p", [element(1, Other)])), $", $]].

anno_line(Anno) ->
    case erl_anno:line(Anno) of
        L when is_integer(L) -> L;
        {L, _} -> L
    end.

loc_line({L, _}) when is_integer(L) -> L;
loc_line(L) when is_integer(L) -> L;
loc_line(_) -> 1.

error_line({Loc, _, _}) -> loc_line(Loc);
error_line(_) -> 1.

read_utf8(File) ->
    case file:read_file(File) of
        {ok, Bin} ->
            case unicode:characters_to_list(Bin, utf8) of
                Src when is_list(Src) -> {ok, Src};
                _ -> error
            end;
        _ -> error
    end.

emit_obj(Fields) ->
    Parts = [lists:flatten(field_json(K, V)) || {K, V} <- Fields],
    io:put_chars([${, string:join(Parts, ","), $}, $\n]).

field_json("expect_later_forms", N) when is_integer(N) ->
    ["\"expect_later_forms\":", integer_to_list(N)];
field_json("tree_raw", Tree) when Tree =/= undefined ->
    ["\"tree\":", Tree];
field_json("forms_raw", Forms) ->
    ["\"forms\":[", string:join([form_object(F) || F <- Forms], ","), $]];
field_json(Key, Val) when is_list(Val); is_binary(Val) ->
    [$", Key, $", $:, $", escape(Val), $"].

form_object(#{parse := P, category := C, line := L} = F) ->
    Epp = case maps:get(epp, F, undefined) of
              undefined -> "";
              E -> [",", "\"epp\":\"", atom_to_list(E), "\""]
          end,
    [${,
     "\"parse\":\"", atom_to_list(P), "\",",
     "\"category\":\"", escape(C), "\",",
     "\"line\":", integer_to_list(L),
     Epp,
     $}].

escape(B) when is_binary(B) -> escape(binary_to_list(B));
escape(S) when is_list(S) -> lists:flatten([escape_char(C) || C <- lists:flatten(S)]).

escape_char($\\) -> "\\\\";
escape_char($") -> "\\\"";
escape_char($\n) -> "\\n";
escape_char($\r) -> "\\r";
escape_char($\t) -> "\\t";
escape_char($\b) -> "\\b";
escape_char($\f) -> "\\f";
escape_char(C) when is_integer(C), C < 32 -> io_lib:format("\\u~4.16.0b", [C]);
escape_char(C) -> C.
