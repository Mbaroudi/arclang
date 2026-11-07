# ArcLang Studio: the whole chain in one image, local or remote

ArcLang Studio is one Docker image with everything ArcLang drives:

- **VS Code** with the ArcLang extension: in the browser through
  openvscode-server, or your own VS Code attached to the container;
- the **compiler** (`arclang`), of the image's version;
- **OpenModelica** 1.27.1 with the Modelica Standard Library 4.1.0, for
  simulations, twins and plants;
- the **C compilers** (GCC, Clang) and cppcheck's MISRA addon, for SIL;
- **Zephyr** and **Renode**, for PIL;
- **GHDL**, **Yosys** and **nextpnr-ecp5**, for HDL and synthesis;
- **LiteX** and **Verilator**, for the SoC, plus openFPGALoader for a
  board.

The image is the same wherever it runs: on your computer, on a team's
server, or in GitHub Codespaces. A model checked, simulated, run in the
loop or synthesised gives the same verdicts in all three places. It is
published at each release for amd64 and arm64:

```text
ghcr.io/mbaroudi/arclang-studio:4.11.0      (and :latest)
```

It is large, several GB, because the hardware toolchains are. It is
pulled once.

## Three ways to use it

**1. VS Code in the browser, on your computer.**

```sh
docker run --rm -p 127.0.0.1:3000:3000 -v "$PWD":/home/arclang/workspace ghcr.io/mbaroudi/arclang-studio:4.11.0
```

The container prints the address to open, with a connection token. It
opens the mounted directory, or the examples when nothing is mounted. Every
command of the extension, the chat tools and the terminal run inside the
container, with all the tools.

**2. Your own VS Code, attached to the container.** Copy
[`.devcontainer/devcontainer.json`](https://github.com/Mbaroudi/arclang/blob/main/.devcontainer/devcontainer.json)
into the `.devcontainer/` directory of your project. Then run **Dev
Containers: Reopen in Container**. You keep your VS Code, your keybindings
and your settings, and the project and the tools are in the container. The
same file opens the project in **GitHub Codespaces**.

**3. On a server, for a remote team.** Run the image as in 1, on the
server. Its port should not be public as it is: put it behind something
that authenticates, such as a reverse proxy with your company's login or
a VPN. In that case, set `ARCLANG_STUDIO_TOKEN` to a secret of your own,
or to `none` when the proxy already authenticates. Each person gets the
same environment as on a laptop.

Serve it over **HTTPS** (the proxy terminates TLS). The browser gives VS
Code's webviews (the workbench, the diagrams, the tables) the crypto they
need only on `https://` or `localhost`: over plain `http://` from another
machine, the editor opens but those views stay blank.

The server does not ask whether you trust the folder: the container is one
user's, behind its token, and VS Code trusts a dev container the same way.

## The compiler on your computer, the tools in the image

Without the image open, the `arclang` of your computer (and the VS Code
extension that calls it) still runs `pil`, `hdl` and `soc` when the
computer lacks their tools. It runs that command in ArcLang Studio of its
own version:

1. It pulls the image the first time.
2. It mounts the model's directory, as your user.
3. It writes the verdict beside the model, as if the command had run on
   your computer.

The model, the editor and the cache stay on your computer. Only the tools
come from the image. Set `ARCLANG_EMBEDDED_DOCKER` to change this:

- `off`: everything runs on this computer;
- an image name: that image is used instead;
- `dev`: the images built from the sources by `tools/embedded/*/run.sh`,
  with the compiler of their target volume, for work on the compiler
  itself.

## Settings of the image

| Variable | Default | What it does |
|---|---|---|
| `ARCLANG_STUDIO_TOKEN` | a random token, printed | the connection token of the server; `none` turns it off |
| `ARCLANG_STUDIO_PORT` | `3000` | the port of the server in the container |
| `ARCLANG_STUDIO_FOLDER` | `/home/arclang/workspace` | the folder opened (the examples when it is empty) |

`docker run ... arclang-studio arclang check model.arc` runs the compiler
alone, and `... arclang-studio bash` opens a shell. The user is `arclang`
(uid 1000), not root.

## Building it

```sh
tools/studio/build.sh            # the chain's images when missing, then arclang-studio:dev
```

The first time takes an hour or two: GHDL is compiled, and Zephyr, LiteX
and the toolchains are fetched. After that, building the studio alone takes
minutes. The release workflow (`.github/workflows/studio.yml`) does the
same on an amd64 and an arm64 runner. Before it publishes, it runs SIL and
HDL in each image. The compiler and the extension in the image are built
from the same sources.

What is not in it:

- **Simulink**, which enters as an FMU: a plant's `fmu:`, FMI 2.0 or 3.0;
- **Capella**;
- **a hosted instance on arclang.fr**. The image runs on your computer,
  your server or Codespaces.
