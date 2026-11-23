import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const serverOptions: ServerOptions = {
    command: "ueas",
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ueas" }],
  };

  client = new LanguageClient("ueasLsp", "UEAS v2.0 Language Server", serverOptions, clientOptions);
  void client.start();

  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("ueas", {
      createDebugAdapterDescriptor(): vscode.DebugAdapterDescriptor {
        return new vscode.DebugAdapterExecutable("ueas", ["dap"]);
      },
    }),
  );
}

export function deactivate(): Thenable<void> | undefined {
  return client ? client.stop() : undefined;
}
