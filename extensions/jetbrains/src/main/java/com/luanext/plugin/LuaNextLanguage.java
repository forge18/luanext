package com.luanext.plugin;

import com.intellij.lang.Language;

/**
 * Language definition for LuaNext
 */
public class LuaNextLanguage extends Language {
    public static final LuaNextLanguage INSTANCE = new LuaNextLanguage();

    private LuaNextLanguage() {
        super("LuaNext");
    }

    @Override
    public String getDisplayName() {
        return "LuaNext";
    }

    @Override
    public boolean isCaseSensitive() {
        return true;
    }
}
