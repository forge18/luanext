// LuaNext Custom Theme Script

(function() {
    'use strict';

    // Initialize version selector styling
    function initVersionSelector() {
        const selector = document.getElementById('version-selector');
        if (selector) {
            selector.style.position = 'fixed';
            selector.style.bottom = '20px';
            selector.style.right = '20px';
            selector.style.zIndex = '1000';
            selector.style.padding = '8px';
            selector.style.borderRadius = '4px';
            selector.style.backgroundColor = 'var(--code-bg-color)';
            selector.style.border = '1px solid var(--border-color)';
        }

        const picker = document.getElementById('version-picker');
        if (picker) {
            picker.style.padding = '6px 10px';
            picker.style.borderRadius = '3px';
            picker.style.border = '1px solid var(--border-color)';
            picker.style.backgroundColor = 'var(--bg-color)';
            picker.style.color = 'var(--fg-color)';
            picker.style.fontSize = '0.9em';
            picker.style.cursor = 'pointer';
        }
    }

    // Apply custom theme on page load
    function applyCustomTheme() {
        const root = document.documentElement;
        const htmlClass = root.className || '';

        // Ensure theme variables are loaded
        const computedStyle = getComputedStyle(root);
        const currentTheme = htmlClass.includes('dark') ? 'dark' : 'light';

        // Log for debugging (remove in production)
        console.log('LuaNext theme applied:', currentTheme);
    }

    // Handle theme switching
    function setupThemeSwitcher() {
        const toggle = document.querySelector('.theme-toggle');
        if (toggle) {
            toggle.addEventListener('click', function() {
                setTimeout(applyCustomTheme, 100);
            });
        }
    }

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            initVersionSelector();
            applyCustomTheme();
            setupThemeSwitcher();
        });
    } else {
        // DOM is already loaded
        initVersionSelector();
        applyCustomTheme();
        setupThemeSwitcher();
    }

    // Handle visibility changes (e.g., when user returns to tab)
    document.addEventListener('visibilitychange', function() {
        if (!document.hidden) {
            applyCustomTheme();
        }
    });
})();
