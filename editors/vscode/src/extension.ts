import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('LuaNext extension is now active');

    // Start the language server
    startLanguageServer();

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('luanext.restartServer', async () => {
            await restartLanguageServer();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('luanext.showOutputChannel', () => {
            client?.outputChannel.show();
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

async function startLanguageServer() {
    const config = vscode.workspace.getConfiguration('luanext');
    const serverPath = config.get<string>('server.path', 'luanext-lsp');

    // Define the server options
    const serverOptions: ServerOptions = {
        command: serverPath,
        args: [],
        transport: TransportKind.stdio,
        options: {
            env: process.env
        }
    };

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for LuaNext documents
        documentSelector: [
            { scheme: 'file', language: 'luanext' },
            { scheme: 'untitled', language: 'luanext' }
        ],
        synchronize: {
            // Notify the server about file changes to '.luax' files in the workspace
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.luax')
        },
        outputChannelName: 'LuaNext Language Server',
        traceOutputChannel: vscode.window.createOutputChannel('LuaNext Language Server Trace'),
        revealOutputChannelOn: 2, // RevealOutputChannelOn.Error
        initializationOptions: {
            // Pass configuration to the server
            checkOnSave: config.get('compiler.checkOnSave', true),
            strictNullChecks: config.get('compiler.strictNullChecks', true),
            formatEnable: config.get('format.enable', true),
            formatIndentSize: config.get('format.indentSize', 4),
            inlayHintsTypeHints: config.get('inlayHints.typeHints', true),
            inlayHintsParameterHints: config.get('inlayHints.parameterHints', true)
        }
    };

    // Create the language client
    client = new LanguageClient(
        'luanext',
        'LuaNext Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (and server)
    try {
        await client.start();
        vscode.window.showInformationMessage('LuaNext Language Server started successfully');
    } catch (error) {
        vscode.window.showErrorMessage(
            `Failed to start LuaNext Language Server: ${error}`
        );
        console.error('Failed to start language server:', error);
    }
}

async function restartLanguageServer() {
    if (client) {
        vscode.window.showInformationMessage('Restarting LuaNext Language Server...');
        await client.stop();
        client = undefined;
    }
    await startLanguageServer();
}
