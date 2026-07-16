import fs from "node:fs";
import openapiTS, { astToString } from "openapi-typescript";

type Operation = {
    method: string;
    path: string,
    name: string;
}

async function main() {
    const specUrl = new URL("../frontend-common/openapi.json", import.meta.url);

    const astWithEnum = await openapiTS(specUrl, {
        makePathsEnum: true,
        rootTypesNoSchemaPrefix: true,
        rootTypes: true,
        defaultNonNullable: false,
    });
    const contents = astToString(astWithEnum);


    const astWithoutEnum = await openapiTS(specUrl, {
        makePathsEnum: false,
        rootTypesNoSchemaPrefix: true,
        rootTypes: true,
        defaultNonNullable: false,
    });
    const contentsWithoutEnum = astToString(astWithoutEnum);


    const operationsIter = contents.matchAll(/(get|post|delete|options|patch|put): operations\["(\w+)"\];/gm)

    const operations: Operation[] = [];

    for (const match of operationsIter) {

        const pathRE = new RegExp(`${match[2]} = "(\\S+)"`, "s");
        const pathMatch = contents.match(pathRE);

        const path = pathMatch?.[1] || "";
        if (!path) {
            throw new Error(`Could not find path for operation ${match[2]}`);
        }

        operations.push({
            method: match[1],
            name: match[2],
            path: path,
        });
    }

    const fullContent = "// This file is auto-generated. Do not edit manually. (see scripts/generate-webapi.sh)\n"
        + contentsWithoutEnum
        + "\n\nexport const operationsMap = " + JSON.stringify(operations, null, 4);
    fs.writeFileSync(new URL("../frontend-common/src/generated/api.ts", import.meta.url), fullContent);

    console.log("Done", operations);
}

main();
