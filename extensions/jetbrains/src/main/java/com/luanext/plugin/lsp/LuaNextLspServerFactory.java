package com.luanext.plugin.lsp;

import com.intellij.openapi.project.Project;
import com.redhat.devtools.lsp4ij.server.ProcessStreamConnectionProvider;
import org.jetbrains.annotations.NotNull;

import java.util.Arrays;
import java.util.List;

/**
 * Factory for creating LuaNext LSP server instances
 */
public class LuaNextLspServerFactory {

    @NotNull
    public ProcessStreamConnectionProvider createConnectionProvider(@NotNull Project project) {
        // Get the LSP server command from settings
        String serverPath = getLspServerPath(project);

        List<String> commands = Arrays.asList(serverPath);

        return new ProcessStreamConnectionProvider(commands, "luanext-lsp");
    }

    private String getLspServerPath(Project project) {
        // TODO: Get from project settings
        // For now, assume luanext-lsp is in PATH
        return "luanext-lsp";
    }
}
