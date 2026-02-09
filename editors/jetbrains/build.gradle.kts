plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.9.21"
    id("org.jetbrains.intellij") version "1.16.1"
}

group = "com.luanext"
version = "0.1.0"

repositories {
    mavenCentral()
}

// Configure Gradle IntelliJ Plugin
intellij {
    version.set("2023.1.5")
    type.set("IC") // Target IDE Platform (IC = IntelliJ IDEA Community)

    plugins.set(listOf(
        "com.redhat.devtools.lsp4ij:0.0.2" // LSP support
    ))
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib")
    testImplementation("junit:junit:4.13.2")
}

tasks {
    // Set the JVM compatibility versions
    withType<JavaCompile> {
        sourceCompatibility = "17"
        targetCompatibility = "17"
    }
    withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }

    patchPluginXml {
        sinceBuild.set("231")
        untilBuild.set("241.*")

        // Plugin description
        pluginDescription.set("""
            <p>Language support for LuaNext - a statically typed dialect of Lua with TypeScript-inspired syntax.</p>

            <h3>Features:</h3>
            <ul>
                <li>Syntax highlighting for .luax files</li>
                <li>LSP integration with luanext-lsp</li>
                <li>Auto-completion and code navigation</li>
                <li>Real-time type checking and diagnostics</li>
                <li>Code formatting</li>
                <li>Quick fixes and refactoring support</li>
            </ul>

            <h3>Requirements:</h3>
            <ul>
                <li>luanext-lsp language server executable in PATH</li>
            </ul>
        """.trimIndent())

        changeNotes.set("""
            <h3>0.1.0</h3>
            <ul>
                <li>Initial release</li>
                <li>Basic syntax highlighting</li>
                <li>LSP integration</li>
                <li>File type association for .luax files</li>
            </ul>
        """.trimIndent())
    }

    signPlugin {
        certificateChain.set(System.getenv("CERTIFICATE_CHAIN"))
        privateKey.set(System.getenv("PRIVATE_KEY"))
        password.set(System.getenv("PRIVATE_KEY_PASSWORD"))
    }

    publishPlugin {
        token.set(System.getenv("PUBLISH_TOKEN"))
    }

    test {
        useJUnit()
    }
}
