# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.MixProject do
  use Mix.Project

  @app :graphlaw_ex
  @version "26.11.25"
  @source_url "https://github.com/seanchatmangpt/graphlaw"

  def project do
    [
      app: @app,
      version: @version,
      elixir: "~> 1.18",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      description: """
      BEAM bridge for the graphlaw WASM law kernel. The BEAM side holds only
      the wasm byte-slice; all law admission happens inside the module
      (deterministic turnstile boundary).
      """,
      package: package(),
      deps: deps(),
      source_url: @source_url,
      docs: [
        main: "readme",
        extras: [
          "README.md",
          "guides/getting_started.md",
          "guides/capabilities.md",
          "guides/refusals_and_errors.md",
          "guides/releases.md",
          "guides/operations.md",
          "guides/architecture.md",
          "CHANGELOG.md",
          "guides/ash_r2rml_migration.md",
          "graphlaw_ex.livemd"
        ],
        groups_for_extras: [
          Tutorials: ["guides/getting_started.md", "graphlaw_ex.livemd"],
          "How-to": [
            "guides/operations.md",
            "guides/releases.md",
            "guides/ash_r2rml_migration.md"
          ],
          Reference: [
            "guides/capabilities.md",
            "guides/refusals_and_errors.md",
            "CHANGELOG.md"
          ],
          Explanation: ["guides/architecture.md"]
        ],
        source_ref: "v#{@version}",
        source_url_pattern: "#{@source_url}/blob/v#{@version}/graphlaw_ex/%{path}#L%{line}",
        groups_for_modules: [
          Config: [GraphlawEx.WasmConfig],
          Telemetry: [GraphlawEx.Telemetry],
          Errors: ~r/^GraphlawEx\.Error/,
          Capabilities: ~r/^GraphlawEx\.Capability/,
          Runtime: [
            GraphlawEx.Host,
            GraphlawEx.Pool,
            GraphlawEx.Application,
            GraphlawEx.EngineLoad,
            GraphlawEx.KillSwitch
          ],
          "Authority and refusals": [GraphlawEx.Authority, GraphlawEx.Refusal, GraphlawEx.ABI],
          Results: ~r/^GraphlawEx\.Result/,
          "Mix tasks": ~r/^Mix\.Tasks/
        ]
      ],
      dialyzer: [
        plt_add_apps: [:mix, :ex_unit],
        plt_local_path: "_build/plts",
        plt_core_path: "_build/plts",
        flags: [:error_handling]
      ],
      aliases: aliases()
    ]
  end

  defp aliases do
    [
      gates: [
        "format --check-formatted",
        "compile --warnings-as-errors",
        "test",
        "graphlaw_ex.verify"
      ],
      ci: [
        "format --check-formatted",
        "compile --warnings-as-errors",
        "test",
        "graphlaw_ex.verify",
        "dialyzer",
        "hex.build"
      ]
    ]
  end

  def cli do
    [preferred_envs: [gates: :test, ci: :test]]
  end

  def application do
    [extra_applications: [:logger, :crypto], mod: {GraphlawEx.Application, []}]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp package do
    [
      # Option 2: the wasm artifact exceeds Hex's 8 MB limit and is NOT packaged.
      # Obtain it with `mix graphlaw_ex.fetch` (GitHub Release v26.11.25) or
      # GRAPHLAW_WASM_PATH. Order: GRAPHLAW_WASM_PATH -> staged priv/graphlaw -> fetch.
      files: ~w(lib guides mix.exs README.md CHANGELOG.md graphlaw_ex.livemd LICENSE* LICENSES
           priv/graphlaw/MANIFEST.json priv/graphlaw/graphlaw.wasm.blake3
           priv/graphlaw/graphlaw.wasm.sha256),
      licenses: ["MIT"],
      links: %{
        "GitHub" => @source_url,
        "Changelog" => @source_url <> "/blob/main/graphlaw_ex/CHANGELOG.md"
      }
    ]
  end

  defp deps do
    [
      # wasmex: Elixir bindings for wasmtime.
      {:wasmex, "~> 0.18.1"},
      {:jason, "~> 1.4"},
      {:telemetry, "~> 1.2"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false}
    ]
  end
end
