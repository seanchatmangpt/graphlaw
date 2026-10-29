defmodule GraphlawEx.MixProject do
  use Mix.Project

  @app :graphlaw_ex
  @version "0.1.0"

  def project do
    [
      app: @app,
      version: @version,
      elixir: "~> 1.14",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      description: """
      BEAM bridge for the graphlaw WASM law kernel. The BEAM side holds only
      the wasm byte-slice; all law admission happens inside the module
      (deterministic turnstile boundary).
      """,
      package: package(),
      deps: deps()
    ]
  end

  def application do
    [extra_applications: [:logger], mod: {GraphlawEx.Application, []}]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp package do
    [files: ~w(lib mix.exs README.md), licenses: ["Apache-2.0"], links: %{"GitHub" => "https://github.com/seanchatmangpt/graphlaw"}]
  end

  defp deps do
    [
      # wasmex: Elixir bindings for wasmtime; 0.9.x is the current stable line.
      {:wasmex, "~> 0.15.1"},
      {:jason, "~> 1.4"},
      {:telemetry, "~> 1.2"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end
end
