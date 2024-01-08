import { Application, TSConfigReader, TypeDocReader } from "typedoc";

async function main() {
    const app = await Application.bootstrapWithPlugins({
        // typedoc options here
        entryPoints: ["../src/ts/docs_index.ts"],
        tsconfig: "../src/ts/tsconfig.json",
        readme: "src/README.md",
        name: "Botloader",
        excludePrivate: true,
        excludeProtected: true,
        excludeInternal: true,
    });

    // If you want TypeDoc to load tsconfig.json / typedoc.json files
    // app.options.addReader(new TSConfigReader());
    // app.options.addReader(new TypeDocReader());

    // app.bootstrap({
    //     // typedoc options here
    //     entryPoints: ["../src/ts/docs_index.ts"],
    //     tsconfig: "../src/ts/tsconfig.json",
    //     readme: "src/README.md",
    //     name: "Botloader",
    //     excludePrivate: true,
    //     excludeProtected: true,
    //     excludeInternal: true,
    // });

    const project = await app.convert();

    if (project) {
        // Project may not have converted correctly
        const outputDir = "docs";

        // Rendered docs
        await app.generateDocs(project, outputDir);
    }
}

main().catch(console.error);