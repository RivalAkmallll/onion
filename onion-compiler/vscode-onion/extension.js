const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

function activate() {
  const configured = vscode.workspace.getConfiguration("onion").get("serverPath");
  const command = configured && configured.length > 0 ? configured : "onion-lsp";

  const serverOptions = { command, args: [] };
  const clientOptions = {
    documentSelector: [{ language: "onion" }],
  };

  client = new LanguageClient(
    "onion-lsp",
    "Onion Language Server",
    serverOptions,
    clientOptions
  );

  client.start().catch((err) => {
    vscode.window.showErrorMessage(
      `Onion: cannot start "${command}". Set "onion.serverPath" in settings. (${err.message})`
    );
  });
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };