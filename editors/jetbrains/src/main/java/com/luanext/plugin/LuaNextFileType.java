package com.luanext.plugin;

import com.intellij.openapi.fileTypes.LanguageFileType;
import com.intellij.openapi.util.IconLoader;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import javax.swing.*;

/**
 * File type definition for LuaNext (.luax files)
 */
public class LuaNextFileType extends LanguageFileType {
    public static final LuaNextFileType INSTANCE = new LuaNextFileType();

    private LuaNextFileType() {
        super(LuaNextLanguage.INSTANCE);
    }

    @NotNull
    @Override
    public String getName() {
        return "LuaNext";
    }

    @NotNull
    @Override
    public String getDescription() {
        return "LuaNext source file";
    }

    @NotNull
    @Override
    public String getDefaultExtension() {
        return "luax";
    }

    @Nullable
    @Override
    public Icon getIcon() {
        // Load icon from resources (create a simple icon or use default)
        return IconLoader.findIcon("/fileTypes/luanext.svg", LuaNextFileType.class);
    }
}
