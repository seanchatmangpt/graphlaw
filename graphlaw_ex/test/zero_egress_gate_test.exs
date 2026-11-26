# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.ZeroEgressGateTest do
  @moduledoc """
  Gate 4: zero network egress and zero pure-Elixir RDF/SPARQL stacks.

  Scans `lib/` and `mix.exs` for HTTP/socket client references, pure-Elixir
  rdf/sparql dependencies, and `path:` dependencies. Includes an anti-vacuity
  court proving the scanner flags synthetic offenders in a temp dir.
  """
  use ExUnit.Case, async: true

  @root Path.expand("..", __DIR__)

  # Code-level egress references (lib/ and mix.exs).
  @egress_patterns [
    {:tesla, ~r/\bTesla\b|\{:tesla\b/},
    {:req, ~r/\bReq\.|\bReq\b(?!\w)|\{:req\b/},
    {:finch, ~r/\bFinch\b|\{:finch\b/},
    {:mint, ~r/\bMint\b|\{:mint\b/},
    {:httpc, ~r/:httpc\b/},
    {:gen_tcp, ~r/:gen_tcp\b/},
    {:ssl, ~r/:ssl\b/},
    {:inet, ~r/:inet\b/}
  ]

  # The single file allowed to touch :httpc/:ssl/:inets (build-time fetch).
  @fetch_task "lib/mix/tasks/graphlaw_ex.fetch.ex"
  @fetch_only_patterns [{:httpc, ~r/:httpc\b/}, {:ssl, ~r/:ssl\b/}, {:inets, ~r/:inets\b/}]

  # Pure-Elixir RDF/SPARQL dependency declarations.
  @rdf_dep_pattern ~r/\{:(rdf|sparql|sparql_client|shex|shacl|json_ld|rdf_xml|turtle|ex_rdf|linked_data|grax)\s*,/

  # A path: dependency tuple, e.g. {:foo, path: "../foo"}.
  @path_dep_pattern ~r/\{:\w+\s*,[^}]*\bpath:/

  # Code-only patterns, checked after docs and whole-line comments are stripped.
  @code_patterns [
    {:url_literal, ~r{https?://}},
    {:shell_egress, ~r/System\.cmd\(\s*"(?:curl|wget|git)\b/},
    {:os_cmd, ~r/:os\.cmd\b/},
    {:binary_to_term, ~r/\bbinary_to_term\b/},
    {:non_executable_binary_to_term, ~r/\bnon_executable_binary_to_term\b/},
    {:code_eval, ~r/\bCode\.eval_(?:string|quoted|file)\b/},
    {:string_to_atom, ~r/\bString\.to_atom\b/}
  ]

  # Port.open is allowed only in the host (the wasm engine runner).
  @port_pattern {:port_open, ~r/\bPort\.open\b/}
  @port_exempt "lib/graphlaw_ex/host.ex"

  @doc_heredoc ~r/@(?:moduledoc|doc|typedoc)\s+(?:~[sS])?"""\n.*?\n\s*"""/s
  @doc_inline ~r/@(?:moduledoc|doc|typedoc)\s+"[^\n]*"/

  @doc false
  def strip_docs_and_comments(text) do
    text
    |> String.replace(@doc_heredoc, "")
    |> String.replace(@doc_inline, "")
    |> String.split("\n")
    |> Enum.reject(&String.starts_with?(String.trim_leading(&1), "#"))
    |> Enum.join("\n")
  end

  @doc false
  def scan_text(text, kind, rel \\ nil) do
    case kind do
      :lib ->
        code = strip_docs_and_comments(text)
        port = if rel == @port_exempt, do: [], else: [@port_pattern]

        for({name, re} <- @egress_patterns, Regex.match?(re, text), do: name) ++
          for({name, re} <- @code_patterns ++ port, Regex.match?(re, code), do: name)

      :mix ->
        patterns =
          @egress_patterns ++ [{:rdf_dep, @rdf_dep_pattern}, {:path_dep, @path_dep_pattern}]

        for {name, re} <- patterns, Regex.match?(re, text), do: name
    end
  end

  @doc false
  def scan_tree(dir) do
    lib =
      Path.join(dir, "lib/**/*.ex")
      |> Path.wildcard()
      |> Enum.reject(&String.ends_with?(&1, @fetch_task))
      |> Enum.flat_map(&file_hits(&1, :lib))

    mix =
      case File.read(Path.join(dir, "mix.exs")) do
        {:ok, text} -> for h <- scan_text(text, :mix), do: {Path.join(dir, "mix.exs"), h}
        {:error, _} -> []
      end

    lib ++ mix
  end

  defp file_hits(path, kind) do
    rel =
      case String.split(path, "/lib/", parts: 2) do
        [_, tail] -> "lib/" <> tail
        _ -> path
      end

    for h <- scan_text(File.read!(path), kind, rel), do: {path, h}
  end

  test "lib/ and mix.exs contain zero egress, pure-Elixir rdf, or path deps" do
    assert File.exists?(Path.join(@root, "mix.exs"))
    assert Path.wildcard(Path.join(@root, "lib/**/*.ex")) != []
    assert scan_tree(@root) == []
  end

  test "the fetch task is the sole file touching :httpc/:ssl/:inets; runtime lib/graphlaw_ex has none" do
    runtime =
      Path.wildcard(Path.join(@root, "lib/graphlaw_ex/**/*.ex")) ++
        [Path.join(@root, "lib/graphlaw_ex.ex")]

    assert runtime != []

    all = Path.wildcard(Path.join(@root, "lib/**/*.ex"))

    touching =
      for path <- all,
          text = File.read!(path),
          Enum.any?(@fetch_only_patterns, fn {_n, re} -> Regex.match?(re, text) end),
          do: Path.relative_to(path, @root)

    assert touching -- [@fetch_task] == []

    for path <- runtime, File.exists?(path) do
      text = File.read!(path)

      for {name, re} <- @fetch_only_patterns do
        refute Regex.match?(re, text), "#{path} references #{name} in runtime code"
      end
    end
  end

  test "anti-vacuity: scanner flags synthetic offenders in a temp dir" do
    tmp = Path.join(System.tmp_dir!(), "zero_egress_#{System.unique_integer([:positive])}")
    File.mkdir_p!(Path.join(tmp, "lib"))
    on_exit(fn -> File.rm_rf!(tmp) end)

    File.write!(Path.join(tmp, "lib/bad.ex"), """
    defmodule Bad do
      def a, do: Tesla.get("http://x")
      def b, do: Req.get!("http://x")
      def c, do: Finch.build(:get, "http://x")
      def d, do: Mint.HTTP.connect(:http, "x", 80)
      def e, do: :httpc.request("http://x")
      def f, do: :gen_tcp.connect(~c"x", 80, [])
      def g, do: :ssl.connect(~c"x", 443, [])
      def h, do: :inet.gethostbyname(~c"x")
    end
    """)

    # Exemption is path-scoped: an offender at the fetch-task path is skipped,
    # the same body anywhere else is flagged (checked via the :ssl hit above).
    File.mkdir_p!(Path.join(tmp, "lib/mix/tasks"))

    File.write!(
      Path.join(tmp, @fetch_task),
      "defmodule F do\n  def a, do: :httpc.request(~c\"x\")\nend\n"
    )

    File.mkdir_p!(Path.join(tmp, "lib/graphlaw_ex"))

    File.write!(Path.join(tmp, "lib/graphlaw_ex/bad2.ex"), """
    defmodule Bad2 do
      def a, do: System.cmd("curl", ["-O", "x"])
      def b, do: :os.cmd(~c"ls")
      def c, do: Port.open({:spawn, "x"}, [])
    end
    """)

    # Port.open at the host path is exempt.
    File.write!(
      Path.join(tmp, "lib/graphlaw_ex/host.ex"),
      "defmodule Host do\n  def a, do: Port.open({:spawn, \"x\"}, [])\nend\n"
    )

    File.write!(Path.join(tmp, "mix.exs"), """
    defp deps do
      [{:rdf, "~> 2.0"}, {:sparql, "~> 0.3"}, {:local, path: "../local"}]
    end
    """)

    refute Enum.any?(scan_tree(tmp), fn {p, _} -> String.ends_with?(p, "host.ex") end)

    hits = scan_tree(tmp) |> Enum.map(&elem(&1, 1)) |> Enum.uniq() |> Enum.sort()

    assert hits ==
             Enum.sort([
               :url_literal,
               :shell_egress,
               :os_cmd,
               :port_open,
               :tesla,
               :req,
               :finch,
               :mint,
               :httpc,
               :gen_tcp,
               :ssl,
               :inet,
               :rdf_dep,
               :path_dep
             ])

    refute Enum.any?(scan_tree(tmp), fn {p, _} -> String.ends_with?(p, @fetch_task) end)
  end

  test "anti-vacuity: scan_text flags each synthetic offender and ignores docs/comments" do
    assert :url_literal in scan_text(~s|def a, do: "https://evil.example/x"|, :lib)
    assert :shell_egress in scan_text(~s|System.cmd("wget", ["x"])|, :lib)
    assert :shell_egress in scan_text(~s|System.cmd("git", ["clone", "x"])|, :lib)
    assert :os_cmd in scan_text(~s|:os.cmd(~c"curl x")|, :lib)
    assert :port_open in scan_text("Port.open({:spawn, \"x\"}, [])", :lib, "lib/graphlaw_ex/a.ex")
    refute :port_open in scan_text("Port.open({:spawn, \"x\"}, [])", :lib, @port_exempt)
    assert :binary_to_term in scan_text(":erlang.binary_to_term(bin, [:safe])", :lib)

    assert :non_executable_binary_to_term in scan_text(
             "Plug.Crypto.non_executable_binary_to_term(bin)",
             :lib
           )

    assert :code_eval in scan_text("Code.eval_string(x)", :lib)
    assert :code_eval in scan_text("Code.eval_string(x)", :lib, "lib/graphlaw_ex/a.ex")

    assert :code_eval in scan_text(
             "Code.eval_quoted(x)",
             :lib,
             "lib/graphlaw_ex/capability/extension.ex"
           )

    assert :code_eval in scan_text("Code.eval_file(x)", :lib, "lib/graphlaw_ex/a.ex")
    assert :string_to_atom in scan_text("String.to_atom(x)", :lib)

    documented = """
    defmodule D do
      @moduledoc \"\"\"
      See https://example.org/docs and System.cmd("curl", []).
      \"\"\"
      # https://example.org in a comment
      @doc "see http://example.org"
      def a, do: 1
    end
    """

    assert scan_text(documented, :lib) == []
  end

  test "anti-vacuity: clean temp tree yields zero hits" do
    tmp = Path.join(System.tmp_dir!(), "zero_egress_clean_#{System.unique_integer([:positive])}")
    File.mkdir_p!(Path.join(tmp, "lib"))
    on_exit(fn -> File.rm_rf!(tmp) end)
    File.write!(Path.join(tmp, "lib/ok.ex"), "defmodule Ok do\n  def a, do: 1\nend\n")
    File.write!(Path.join(tmp, "mix.exs"), "defp deps, do: [{:jason, \"~> 1.4\"}]\n")
    assert scan_tree(tmp) == []
  end
end
